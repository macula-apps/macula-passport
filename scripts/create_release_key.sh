#!/usr/bin/env bash
# Create the passport's release key, ONCE, and hand it to CI and to its owner (#6).
#
# Every release apk is signed with this key, so a phone that installed one
# version installs the next over it. Lose the key and no phone can ever
# update the app again; leak it and anyone can ship an "update". So:
#   - the key is made here, on the owner's machine, and never printed;
#   - CI gets it as two repository secrets, ANDROID_RELEASE_KEYSTORE_B64 and
#     ANDROID_RELEASE_KEYSTORE_PASSWORD (gh, already logged in);
#   - the owner gets an encrypted backup under a passphrase they choose (gpg,
#     AES-256), proved to decrypt before anything is removed;
#   - the plaintext copy is removed at the end unless the owner says keep.
#
# Usage: scripts/create_release_key.sh
# Needs: keytool (any JDK), gpg, gh with admin on macula-apps/macula-passport.
set -euo pipefail

REPO="macula-apps/macula-passport"
ALIAS="macula-passport"
WORK="${HOME}/macula-passport-release-key"
BACKUP="${HOME}/macula-passport-release-key.tar.gpg"

for tool in keytool gpg gh tar base64 sha256sum; do
    command -v "$tool" >/dev/null || { echo "missing: $tool"; exit 1; }
done
[ ! -e "$WORK" ] || { echo "$WORK already exists: the key was made before. Stop."; exit 1; }
[ ! -e "$BACKUP" ] || { echo "$BACKUP already exists: the key was made before. Stop."; exit 1; }
if gh secret list -R "$REPO" | grep -q '^ANDROID_RELEASE_KEYSTORE_B64'; then
    echo "$REPO already holds a release key secret. Replacing it would break updates. Stop."
    exit 1
fi

# gpg asks for the passphrase through pinentry on this terminal; stdin
# carries the key, so the prompt must not read it.
[ -t 0 ] || { echo "Run this in a terminal: gpg asks for a passphrase."; exit 1; }
GPG_TTY=$(tty)
export GPG_TTY

umask 077
mkdir "$WORK"
head -c 32 /dev/urandom | base64 | tr -d '/+=\n' > "$WORK/password"

echo "1/4 Making the release key (RSA 4096, valid 50 years)..."
keytool -genkeypair -keystore "$WORK/release.p12" -storetype PKCS12 -alias "$ALIAS" \
    -keyalg RSA -keysize 4096 -validity 18250 -dname "CN=Macula Passport, O=Macula" \
    -storepass:file "$WORK/password" -keypass:file "$WORK/password" >/dev/null
CERT=$(keytool -list -v -keystore "$WORK/release.p12" -storepass:file "$WORK/password" -alias "$ALIAS" \
    | grep 'SHA256:' | head -1 | sed 's/.*SHA256: *//')
cat > "$WORK/README.txt" <<EOF
Macula Passport release key ($REPO).
release.p12: PKCS12 keystore, alias $ALIAS. password: its store and key password.
Certificate SHA-256: $CERT
Made $(date -u +%Y-%m-%dT%H:%MZ). Every release apk is signed with it; without it no phone can update the app.
EOF

echo "2/4 Giving it to CI as repository secrets of $REPO..."
base64 -w0 "$WORK/release.p12" | gh secret set ANDROID_RELEASE_KEYSTORE_B64 -R "$REPO"
gh secret set ANDROID_RELEASE_KEYSTORE_PASSWORD -R "$REPO" < "$WORK/password"

echo "3/4 Your encrypted backup. Choose a passphrase and keep it in your password manager."
echo "    gpg asks for it twice here, then once more in step 4 to prove the backup."
tar -C "$WORK" -cf - release.p12 password README.txt \
    | gpg --symmetric --cipher-algo AES256 --output "$BACKUP"

echo "4/4 Proving the backup decrypts (gpg asks for the passphrase once more)..."
gpg --forget-cached-passphrase >/dev/null 2>&1 || true
gpg-connect-agent reloadagent /bye >/dev/null 2>&1 || true
RESTORED=$(gpg --decrypt "$BACKUP" 2>/dev/null | tar -xOf - release.p12 | sha256sum | cut -d' ' -f1)
ORIGINAL=$(sha256sum "$WORK/release.p12" | cut -d' ' -f1)
[ "$RESTORED" = "$ORIGINAL" ] || { echo "The backup did NOT decrypt to the same key. Nothing removed; ask for help."; exit 1; }
echo "    The backup decrypts to the same key."

echo
echo "Done. Certificate SHA-256 (public, it shows in every release's notes):"
echo "    $CERT"
echo "Your backup: $BACKUP"
echo "Copy it to two places you trust (for example your password manager and a USB stick),"
echo "with the passphrase in your password manager."
# No answer (no terminal left, end of input) takes the default: the backup is proven by now.
answer=""
read -r -p "Remove the plaintext key in $WORK now? CI and your backup both hold it. [Y/n] " answer || true
case "${answer:-Y}" in
    [Nn]*) echo "Kept $WORK. Remove it yourself once the backup is stored." ;;
    *)     rm -rf "$WORK"; echo "Removed $WORK." ;;
esac
