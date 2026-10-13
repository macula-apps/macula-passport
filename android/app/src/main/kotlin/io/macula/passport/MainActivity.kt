package io.macula.passport

import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import io.macula.passport.initiate_passport.StartScreen
import io.macula.passport.register_identity_document.IdentityDocument
import io.macula.passport.register_identity_document.IdentityDocumentForm
import io.macula.passport.review_dossier.DossierScreen
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.macula_passport_ffi.FfiClaim
import uniffi.macula_passport_ffi.FfiHolderKind

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent { PassportTheme { Surface(Modifier.fillMaxSize()) { PassportApp(filesDir) } } }
    }
}

/** Where the holder is: every screen the app has, and what it is waiting for. */
private sealed interface Screen {
    data object Opening : Screen
    data class CannotOpen(val why: String) : Screen
    data class Start(val starting: Boolean = false, val failure: String? = null) : Screen
    data class Dossier(val claims: List<FfiClaim>) : Screen
    data class AddDocument(val claims: List<FfiClaim>, val recording: Boolean = false, val failure: String? = null) : Screen
}

@Composable
private fun PassportApp(privateDir: File) {
    var phone by remember { mutableStateOf<PassportOnThisPhone?>(null) }
    var screen by remember { mutableStateOf<Screen>(Screen.Opening) }
    val scope = rememberCoroutineScope()

    LaunchedEffect(privateDir) {
        screen = runCatching {
            withContext(Dispatchers.IO) {
                val opened = PassportOnThisPhone.open(privateDir)
                phone = opened
                when {
                    opened.passport.isInitiated() -> Screen.Dossier(opened.passport.listClaims())
                    else -> Screen.Start()
                }
            }
        }.getOrElse { Screen.CannotOpen(explain(it)) }
    }

    fun start(opened: PassportOnThisPhone) = scope.launch {
        screen = Screen.Start(starting = true)
        screen = runCatching {
            withContext(Dispatchers.IO) {
                opened.passport.initiate(FfiHolderKind.HUMAN, nowSeconds())
                Screen.Dossier(opened.passport.listClaims())
            }
        }.getOrElse { Screen.Start(failure = explain(it)) }
    }

    fun record(opened: PassportOnThisPhone, adding: Screen.AddDocument, document: IdentityDocument) = scope.launch {
        screen = adding.copy(recording = true, failure = null)
        screen = runCatching {
            withContext(Dispatchers.IO) {
                opened.passport.registerIdentityDocument(document.toClaim(nowSeconds()), opened.actingAs)
                Screen.Dossier(opened.passport.listClaims())
            }
        }.getOrElse { adding.copy(recording = false, failure = explain(it)) }
    }

    when (val now = screen) {
        Screen.Opening -> Centered { CircularProgressIndicator() }
        is Screen.CannotOpen -> Centered {
            Text("Your passport could not be opened", style = MaterialTheme.typography.titleLarge)
            Text(now.why, textAlign = TextAlign.Center)
        }
        is Screen.Start -> StartScreen(now.starting, now.failure) { phone?.let(::start) }
        is Screen.Dossier -> DossierScreen(now.claims) { screen = Screen.AddDocument(now.claims) }
        is Screen.AddDocument -> IdentityDocumentForm(
            recording = now.recording,
            recordFailure = now.failure,
            onRecord = { document -> phone?.let { record(it, now, document) } },
            onCancel = { screen = Screen.Dossier(now.claims) },
        )
    }
}

@Composable
private fun Centered(content: @Composable () -> Unit) =
    Column(
        Modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) { content() }

/** Material 3, following the phone's light or dark setting, in its wallpaper colours from Android 12. */
@Composable
private fun PassportTheme(content: @Composable () -> Unit) {
    val dark = isSystemInDarkTheme()
    val context = LocalContext.current
    val colours = when {
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && dark -> dynamicDarkColorScheme(context)
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> dynamicLightColorScheme(context)
        dark -> darkColorScheme()
        else -> lightColorScheme()
    }
    MaterialTheme(colorScheme = colours, content = content)
}
