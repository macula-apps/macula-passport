package io.macula.passport.register_identity_document

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DatePicker
import androidx.compose.material3.DatePickerDialog
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuAnchorType
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.rememberDatePickerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.error
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.unit.dp
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

/**
 * The claim form: the four fields of one identity document. Nothing is recorded
 * until every field checks out; each problem shows under its own field.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun IdentityDocumentForm(
    recording: Boolean,
    recordFailure: String?,
    onRecord: (IdentityDocument) -> Unit,
    onCancel: () -> Unit,
) {
    var type by rememberSaveable { mutableStateOf<DocumentType?>(null) }
    var number by rememberSaveable { mutableStateOf("") }
    var country by rememberSaveable { mutableStateOf<String?>(null) }
    var expiryDay by rememberSaveable { mutableStateOf<Long?>(null) }
    var problems by remember { mutableStateOf(emptyMap<IdentityDocument.Field, String>()) }

    val submit = {
        when (val checked = IdentityDocument.check(type, number, country, expiryDay, todayUtc())) {
            is IdentityDocument.Checked.Valid -> { problems = emptyMap(); onRecord(checked.document) }
            is IdentityDocument.Checked.Problems -> problems = checked.byField
        }
    }

    BackHandler(enabled = !recording, onBack = onCancel)
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Add an identity document") },
                navigationIcon = {
                    IconButton(onClick = onCancel, enabled = !recording) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back to my passport")
                    }
                },
            )
        },
    ) { inner ->
        Column(
            Modifier.fillMaxSize().padding(inner).imePadding().verticalScroll(rememberScrollState()).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Text(
                "It stays on this phone. Nothing is sent anywhere.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            DocumentTypeChoice(type, problems[IdentityDocument.Field.TYPE]) { type = it }
            OutlinedTextField(
                value = number,
                onValueChange = { number = it },
                label = { Text("Document number") },
                singleLine = true,
                isError = IdentityDocument.Field.NUMBER in problems,
                supportingText = { FieldHint(problems[IdentityDocument.Field.NUMBER], "As printed on the document") },
                keyboardOptions = KeyboardOptions(
                    capitalization = KeyboardCapitalization.Characters,
                    autoCorrectEnabled = false,
                    imeAction = ImeAction.Next,
                ),
                modifier = Modifier.fillMaxWidth(),
            )
            CountryChoice(country, problems[IdentityDocument.Field.COUNTRY]) { country = it }
            ExpiryChoice(expiryDay, problems[IdentityDocument.Field.EXPIRY]) { expiryDay = it }
            recordFailure?.let {
                Text(it, color = MaterialTheme.colorScheme.error, modifier = Modifier.semantics { error(it) })
            }
            Button(onClick = submit, enabled = !recording, modifier = Modifier.fillMaxWidth()) {
                when {
                    recording -> CircularProgressIndicator(strokeWidth = 2.dp, modifier = Modifier.padding(2.dp))
                    else -> Text("Record in my passport")
                }
            }
        }
    }
}

@Composable
private fun FieldHint(problem: String?, hint: String) =
    Text(problem ?: hint, color = problem?.let { MaterialTheme.colorScheme.error } ?: MaterialTheme.colorScheme.onSurfaceVariant)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun DocumentTypeChoice(chosen: DocumentType?, problem: String?, onChoose: (DocumentType) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("Kind of document", style = MaterialTheme.typography.labelLarge)
        SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
            DocumentType.entries.forEachIndexed { index, type ->
                SegmentedButton(
                    selected = chosen == type,
                    onClick = { onChoose(type) },
                    shape = SegmentedButtonDefaults.itemShape(index, DocumentType.entries.size),
                ) { Text(type.label, maxLines = 1) }
            }
        }
        problem?.let { Text(it, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall) }
    }
}

/** A country picked from a list that narrows as the holder types: never a free-text code. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CountryChoice(chosen: String?, problem: String?, onChoose: (String) -> Unit) {
    val countries = remember { isoCountries() }
    var expanded by remember { mutableStateOf(false) }
    var query by rememberSaveable { mutableStateOf(chosen?.let(::countryName) ?: "") }
    val matches = countries.filter { (code, name) ->
        query.isBlank() || name.contains(query, ignoreCase = true) || code.equals(query, ignoreCase = true)
    }
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
        OutlinedTextField(
            value = query,
            onValueChange = { query = it; expanded = true },
            label = { Text("Issuing country") },
            singleLine = true,
            isError = problem != null,
            supportingText = { FieldHint(problem, "Start typing to narrow the list") },
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded) },
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = {
                matches.singleOrNull()?.let { (code, name) -> onChoose(code); query = name; expanded = false }
            }),
            modifier = Modifier.fillMaxWidth().menuAnchor(ExposedDropdownMenuAnchorType.PrimaryEditable),
        )
        ExposedDropdownMenu(expanded = expanded && matches.isNotEmpty(), onDismissRequest = { expanded = false }) {
            matches.take(MAX_COUNTRY_MATCHES).forEach { (code, name) ->
                DropdownMenuItem(
                    text = { Text("$name ($code)") },
                    onClick = { onChoose(code); query = name; expanded = false },
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ExpiryChoice(expiryDay: Long?, problem: String?, onChoose: (Long) -> Unit) {
    var picking by remember { mutableStateOf(false) }
    OutlinedTextField(
        value = expiryDay?.let(::formatDay) ?: "",
        onValueChange = {},
        readOnly = true,
        label = { Text("Expiry date") },
        isError = problem != null,
        supportingText = { FieldHint(problem, "The last day the document is valid") },
        trailingIcon = {
            IconButton(onClick = { picking = true }) {
                Icon(Icons.Filled.DateRange, contentDescription = "Pick the expiry date")
            }
        },
        modifier = Modifier.fillMaxWidth(),
    )
    if (picking) {
        val state = rememberDatePickerState(initialSelectedDateMillis = expiryDay?.let { it * 1000 })
        DatePickerDialog(
            onDismissRequest = { picking = false },
            confirmButton = {
                TextButton(
                    onClick = { state.selectedDateMillis?.let { onChoose(it / 1000) }; picking = false },
                    enabled = state.selectedDateMillis != null,
                ) { Text("Use this date") }
            },
            dismissButton = { TextButton(onClick = { picking = false }) { Text("Cancel") } },
        ) { DatePicker(state) }
    }
}

private const val MAX_COUNTRY_MATCHES = 50

/** Every ISO 3166-1 alpha-2 country, named in the phone's language, sorted by name. */
private fun isoCountries(): List<Pair<String, String>> =
    Locale.getISOCountries().map { it to countryName(it) }.sortedBy { it.second }

fun countryName(code: String): String = Locale.Builder().setRegion(code).build().displayCountry

/** Seconds at 00:00 UTC today: the day the date picker and the checks agree on. */
fun todayUtc(): Long = LocalDate.now(ZoneOffset.UTC).atStartOfDay(ZoneOffset.UTC).toEpochSecond()

fun formatDay(daySeconds: Long): String =
    Instant.ofEpochSecond(daySeconds).atZone(ZoneOffset.UTC).toLocalDate()
        .format(DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM))
