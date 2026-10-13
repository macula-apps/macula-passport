package io.macula.passport

import java.io.File
import uniffi.macula_passport_ffi.FfiException
import uniffi.macula_passport_ffi.FfiPassport
import uniffi.macula_passport_ffi.deviceMeshKey
import uniffi.macula_passport_ffi.newHolderId

/**
 * The holder's passport as it lives on this phone, and only here: the dossier
 * file, the holder id next to it and the device's own mesh key, all inside the
 * app's private storage (backup is off in the manifest). Every call does file
 * and SQLite work: call it off the main thread.
 */
class PassportOnThisPhone private constructor(
    val passport: FfiPassport,
    /** The device key every claim desk records as `acting_as`. */
    val actingAs: ByteArray,
) {
    companion object {
        const val DOSSIER_FILE = "passport.db"
        const val HOLDER_ID_FILE = "holder_id"
        const val IDENTITY_DIR = "identity"
        private const val HOLDER_ID_BYTES = 16

        /** Opens the dossier in [privateDir], creating the holder id and the device key on first start. */
        fun open(privateDir: File): PassportOnThisPhone {
            val holderId = holderId(File(privateDir, HOLDER_ID_FILE))
            val actingAs = deviceMeshKey(File(privateDir, IDENTITY_DIR).path)
            val passport = FfiPassport.open(File(privateDir, DOSSIER_FILE).path, holderId)
            return PassportOnThisPhone(passport, actingAs)
        }

        private fun holderId(file: File): ByteArray =
            file.takeIf { it.exists() }?.readBytes()?.also { checkHolderId(file, it) }
                ?: newHolderId().also { writeOnce(file, it) }

        private fun checkHolderId(file: File, bytes: ByteArray) =
            check(bytes.size == HOLDER_ID_BYTES) {
                "${file.name} holds ${bytes.size} bytes, not $HOLDER_ID_BYTES: it was not written by this app"
            }

        // Written to a temporary file and renamed, so a crash never leaves
        // half an id that would open a different, empty dossier next start.
        private fun writeOnce(file: File, bytes: ByteArray) {
            val partial = File(file.parentFile, "${file.name}.partial")
            partial.writeBytes(bytes)
            check(partial.renameTo(file)) { "could not store ${file.name}" }
        }
    }
}

/** Seconds since the epoch: the clock every desk records. */
fun nowSeconds(): Long = System.currentTimeMillis() / 1000

/** What went wrong, in words a holder can act on. */
fun explain(failure: Throwable): String = when (failure) {
    is FfiException.Domain -> "The passport refused this: ${failure.reason}."
    is FfiException.Store ->
        "The passport file could not be read or written (${failure.reason}). Free some storage and try again."
    is FfiException.Identity ->
        "This phone's passport key could not be loaded (${failure.reason}). Your passport is unchanged."
    is FfiException -> "Something could not cross into the passport: ${failure.message}."
    else -> failure.message ?: failure.javaClass.simpleName
}
