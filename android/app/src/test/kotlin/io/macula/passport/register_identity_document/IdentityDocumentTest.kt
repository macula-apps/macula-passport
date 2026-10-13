package io.macula.passport.register_identity_document

import io.macula.passport.register_identity_document.IdentityDocument.Checked
import io.macula.passport.register_identity_document.IdentityDocument.Field
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.macula_passport_ffi.FfiValue

class IdentityDocumentTest {
    private val today = 1_791_763_200L // 2026-10-12 00:00 UTC
    private val nextYear = today + 365 * IdentityDocument.SECONDS_PER_DAY

    private fun valid(checked: Checked) = (checked as Checked.Valid).document
    private fun problems(checked: Checked) = (checked as Checked.Problems).byField

    @Test
    fun a_complete_document_becomes_one_claim_of_its_type() {
        val document = valid(IdentityDocument.check(DocumentType.PASSPORT, "ab 12-34 567", "BE", nextYear, today))
        val claim = document.toClaim(capturedAt = today + 60)

        assertEquals("identity.passport.number", claim.claimType)
        assertEquals(FfiValue.Text("AB1234567"), claim.value)
        assertEquals("BE", claim.issuer)
        assertEquals(today + 60, claim.capturedAt)
        assertEquals(nextYear + IdentityDocument.SECONDS_PER_DAY - 1, claim.expiresAt)
    }

    @Test
    fun every_document_type_reads_back_from_its_claim_type() {
        DocumentType.entries.forEach { assertEquals(it, DocumentType.ofClaimType(it.claimType)) }
        assertNull(DocumentType.ofClaimType("health.blood_type"))
    }

    @Test
    fun an_empty_form_names_every_missing_field() {
        assertEquals(Field.entries.toSet(), problems(IdentityDocument.check(null, " ", null, null, today)).keys)
    }

    @Test
    fun a_number_with_other_characters_is_refused() {
        val found = problems(IdentityDocument.check(DocumentType.ID_CARD, "12/34", "BE", nextYear, today))
        assertEquals(setOf(Field.NUMBER), found.keys)
    }

    @Test
    fun a_number_longer_than_any_document_is_refused() {
        val tooLong = "A".repeat(IdentityDocument.MAX_NUMBER_LENGTH + 1)
        assertTrue(Field.NUMBER in problems(IdentityDocument.check(DocumentType.PASSPORT, tooLong, "BE", nextYear, today)))
    }

    @Test
    fun a_country_must_be_an_alpha_2_code() {
        val found = problems(IdentityDocument.check(DocumentType.PASSPORT, "AB1", "Belgium", nextYear, today))
        assertEquals(setOf(Field.COUNTRY), found.keys)
    }

    @Test
    fun a_document_expiring_today_is_still_valid_and_yesterday_is_not() {
        valid(IdentityDocument.check(DocumentType.DRIVING_LICENCE, "X1", "BE", today, today))
        val found = problems(
            IdentityDocument.check(DocumentType.DRIVING_LICENCE, "X1", "BE", today - IdentityDocument.SECONDS_PER_DAY, today),
        )
        assertEquals(setOf(Field.EXPIRY), found.keys)
    }
}
