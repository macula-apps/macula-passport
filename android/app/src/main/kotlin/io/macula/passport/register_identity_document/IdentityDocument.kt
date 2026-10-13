package io.macula.passport.register_identity_document

import uniffi.macula_passport_ffi.FfiClaim
import uniffi.macula_passport_ffi.FfiValue

/** The identity documents the claim form offers. [slug] is part of the claim type. */
enum class DocumentType(val slug: String, val label: String) {
    PASSPORT("passport", "Passport"),
    ID_CARD("id_card", "Identity card"),
    DRIVING_LICENCE("driving_licence", "Driving licence");

    /**
     * The claim type this document is recorded under, e.g. `identity.passport.number`,
     * so a grant on the prefix `identity.` covers every identity document.
     */
    val claimType: String get() = "$CLAIM_PREFIX$slug$CLAIM_SUFFIX"

    companion object {
        private const val CLAIM_PREFIX = "identity."
        private const val CLAIM_SUFFIX = ".number"

        /** The document type a claim type was recorded under, or null for any other claim. */
        fun ofClaimType(claimType: String): DocumentType? =
            entries.firstOrNull { it.claimType == claimType }
    }
}

/**
 * One identity document as the holder entered it, checked. The four fields map
 * onto one claim: the type names the claim, the number is its value, the
 * issuing country (ISO 3166-1 alpha-2) its issuer, the expiry its expiry.
 */
data class IdentityDocument(
    val type: DocumentType,
    val number: String,
    val issuingCountry: String,
    /** The last day the document is valid, as seconds at 00:00 UTC of that day. */
    val expiryDay: Long,
) {
    /** The document stops being current at the end of its expiry day. */
    val expiresAt: Long get() = expiryDay + SECONDS_PER_DAY - 1

    fun toClaim(capturedAt: Long): FfiClaim =
        FfiClaim(type.claimType, FfiValue.Text(number), issuingCountry, capturedAt, expiresAt)

    companion object {
        const val SECONDS_PER_DAY = 86_400L
        const val MAX_NUMBER_LENGTH = 30
        private val NUMBER = Regex("[A-Z0-9]+")
        private val COUNTRY = Regex("[A-Z]{2}")

        /** Document numbers are compared without spaces or dashes and in capitals. */
        fun normalizeNumber(entered: String): String =
            entered.filterNot { it.isWhitespace() || it == '-' }.uppercase()

        /**
         * Checks what the holder entered against [today] (seconds at 00:00 UTC).
         * Every problem comes back at once, keyed by field, so the form can show
         * each one next to its field.
         */
        fun check(
            type: DocumentType?,
            enteredNumber: String,
            issuingCountry: String?,
            expiryDay: Long?,
            today: Long,
        ): Checked {
            val number = normalizeNumber(enteredNumber)
            val problems = buildMap {
                if (type == null) put(Field.TYPE, "Choose the kind of document.")
                numberProblem(number)?.let { put(Field.NUMBER, it) }
                if (issuingCountry == null || !COUNTRY.matches(issuingCountry)) {
                    put(Field.COUNTRY, "Choose the country that issued the document.")
                }
                expiryProblem(expiryDay, today)?.let { put(Field.EXPIRY, it) }
            }
            return when {
                problems.isNotEmpty() -> Checked.Problems(problems)
                else -> Checked.Valid(IdentityDocument(type!!, number, issuingCountry!!, expiryDay!!))
            }
        }

        private fun numberProblem(number: String): String? = when {
            number.isEmpty() -> "Enter the document number as printed on it."
            number.length > MAX_NUMBER_LENGTH ->
                "That is longer than any document number ($MAX_NUMBER_LENGTH characters at most). Check for a pasted extra."
            !NUMBER.matches(number) -> "Use only the letters and digits printed on the document."
            else -> null
        }

        private fun expiryProblem(expiryDay: Long?, today: Long): String? = when {
            expiryDay == null -> "Pick the expiry date printed on the document."
            expiryDay < today -> "This document has expired. Record a document that is still valid."
            else -> null
        }
    }

    enum class Field { TYPE, NUMBER, COUNTRY, EXPIRY }

    sealed interface Checked {
        data class Valid(val document: IdentityDocument) : Checked
        data class Problems(val byField: Map<Field, String>) : Checked
    }
}
