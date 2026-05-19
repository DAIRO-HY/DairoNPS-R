package cn.dairo.npc.extension
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

data class ExtraColors(
    val bgPrimary: Color,
    val statusLabelFail: Color,
    val statusLabelSuccess: Color,
)

val LocalExtraColors = staticCompositionLocalOf {
    ExtraColors(
        bgPrimary = Color.Unspecified,
        statusLabelFail = Color.Unspecified,
        statusLabelSuccess = Color.Unspecified,
    )
}

val MaterialTheme.extraColors: ExtraColors
    @Composable
    get() = LocalExtraColors.current