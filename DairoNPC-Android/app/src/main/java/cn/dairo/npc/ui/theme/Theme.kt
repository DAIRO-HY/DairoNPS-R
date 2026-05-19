package cn.dairo.npc.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.sp
import cn.dairo.npc.extension.ExtraColors
import cn.dairo.npc.extension.LocalExtraColors

private val LightExtraColors = ExtraColors(
    bgPrimary = Color(0xFF173C6B),
    statusLabelFail = Color(0xFFDC3545),
    statusLabelSuccess = Color(0xFF198754),
)

private val DarkExtraColors = ExtraColors(
    bgPrimary = Color(0xFF002745),
    statusLabelFail = Color(0xFFC22E3D),
    statusLabelSuccess = Color(0xFF157247),
)

//@Composable
//fun MyApplicationTheme(
//    darkTheme: Boolean = isSystemInDarkTheme(),
//    // Dynamic color is available on Android 12+
//    dynamicColor: Boolean = true,
//    content: @Composable () -> Unit
//) {
//    val colorScheme = when {
//        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
//            val context = LocalContext.current
//            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
//        }
//
//        darkTheme -> DarkColorScheme
//        else -> LightColorScheme
//    }
//
//    MaterialTheme(
//        colorScheme = colorScheme,
//        typography = Typography,
//        content = content
//    )
//}

@Composable
fun AppTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    val extraColors =
        if (darkTheme) DarkExtraColors
        else LightExtraColors

    CompositionLocalProvider(
        LocalExtraColors provides extraColors
    ) {
        MaterialTheme(
            typography = Typography(

                //默认字体大小,也就是在没有设置字体大小时的大小
                bodyLarge = TextStyle(
                    fontSize = 14.sp
                ),
                bodyMedium = TextStyle(
                    fontSize = 12.sp
                ),
                bodySmall = TextStyle(
                    fontSize = 10.sp
                ),

                //标题栏字体大小
                titleLarge = TextStyle(
                    fontSize = 20.sp
                ),

                //默认按钮字体大小
                labelLarge = TextStyle(
                    fontSize = 12.sp
                )
            ),
            colorScheme = if (darkTheme) darkColorScheme() else lightColorScheme(),
            content = content
        )
    }
}