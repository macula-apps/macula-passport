package io.macula.passport.review_dossier

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import io.macula.passport.register_identity_document.DocumentType
import io.macula.passport.register_identity_document.IdentityDocument
import io.macula.passport.register_identity_document.countryName
import io.macula.passport.register_identity_document.formatDay
import uniffi.macula_passport_ffi.FfiClaim
import uniffi.macula_passport_ffi.FfiValue

const val ADD_DOCUMENT = "Add document"

/** The dossier as replayed from the file on this phone: every claim it holds. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DossierScreen(claims: List<FfiClaim>, onAddDocument: () -> Unit) {
    Scaffold(
        topBar = { TopAppBar(title = { Text("My passport") }) },
        floatingActionButton = {
            // The button's label is set on the button itself: its text did not reach the
            // accessibility tree, so a screen reader announced an unnamed button (#5).
            ExtendedFloatingActionButton(
                onClick = onAddDocument,
                icon = { Icon(Icons.Filled.Add, contentDescription = null) },
                text = { Text(ADD_DOCUMENT) },
                modifier = Modifier.semantics { contentDescription = ADD_DOCUMENT },
            )
        },
    ) { inner ->
        when {
            claims.isEmpty() -> EmptyDossier(Modifier.padding(inner))
            else -> LazyColumn(
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
                modifier = Modifier.padding(inner),
            ) { items(claims.sortedByDescending { it.capturedAt }) { ClaimCard(it) } }
        }
    }
}

@Composable
private fun EmptyDossier(modifier: Modifier) {
    Column(
        modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("Your passport is started", style = MaterialTheme.typography.titleLarge)
        Text(
            "Add your first identity document: a passport, identity card or driving licence.",
            style = MaterialTheme.typography.bodyLarge,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(top = 8.dp),
        )
    }
}

@Composable
private fun ClaimCard(claim: FfiClaim) {
    val document = DocumentType.ofClaimType(claim.claimType)
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(document?.label ?: claim.claimType, style = MaterialTheme.typography.titleMedium)
            Text(shown(claim.value), style = MaterialTheme.typography.bodyLarge)
            claim.issuer?.let { Text("Issued by ${issuerName(it)}", style = MaterialTheme.typography.bodyMedium) }
            claim.expiresAt?.let {
                val day = it - (IdentityDocument.SECONDS_PER_DAY - 1)
                Text("Valid until ${formatDay(day)}", style = MaterialTheme.typography.bodyMedium)
            }
            Text(
                "Recorded ${formatDay(claim.capturedAt)}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

// The form records a country code; a claim from elsewhere may name its issuer in words.
private fun issuerName(issuer: String): String =
    issuer.takeIf { COUNTRY_CODE.matches(it) }?.let(::countryName) ?: issuer

private val COUNTRY_CODE = Regex("[A-Z]{2}")

private fun shown(value: FfiValue): String = when (value) {
    is FfiValue.Text -> value.v1
    is FfiValue.Int -> value.v1.toString()
    is FfiValue.Float -> value.v1.toString()
    is FfiValue.Bytes -> "${value.v1.size} bytes"
    FfiValue.Null -> "(empty)"
}
