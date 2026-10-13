package io.macula.passport.initiate_passport

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.error
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp

/** First start: the holder starts their passport. Nothing exists before this. */
@Composable
fun StartScreen(starting: Boolean, startFailure: String?, onStart: () -> Unit) {
    Column(
        Modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("Your passport", style = MaterialTheme.typography.headlineMedium)
        Text(
            "Keep your identity documents in a passport that lives only on this phone. " +
                "No account, no cloud copy, nothing sent anywhere.",
            style = MaterialTheme.typography.bodyLarge,
            textAlign = TextAlign.Center,
        )
        startFailure?.let {
            Text(it, color = MaterialTheme.colorScheme.error, textAlign = TextAlign.Center,
                modifier = Modifier.semantics { error(it) })
        }
        Button(onClick = onStart, enabled = !starting, modifier = Modifier.fillMaxWidth()) {
            when {
                starting -> CircularProgressIndicator(strokeWidth = 2.dp, modifier = Modifier.padding(2.dp))
                else -> Text("Start my passport")
            }
        }
    }
}
