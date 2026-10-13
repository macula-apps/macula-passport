package io.macula.passport.review_dossier

import androidx.compose.ui.test.assertHasClickAction
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.performClick
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/** What a screen reader is told on the dossier screen (#5). */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class DossierScreenAccessibilityTest {
    @get:Rule
    val compose = createComposeRule()

    @Test
    fun theAddDocumentButtonIsNamedAndActs() {
        var added = 0
        compose.setContent { DossierScreen(claims = emptyList(), onAddDocument = { added++ }) }
        compose.onNodeWithContentDescription(ADD_DOCUMENT).assertHasClickAction().performClick()
        assertEquals(1, added)
    }
}
