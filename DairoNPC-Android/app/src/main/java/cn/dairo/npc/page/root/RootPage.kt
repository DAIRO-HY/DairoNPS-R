package cn.dairo.npc.page.root

import android.app.Application
import androidx.compose.foundation.layout.Spacer
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import cn.dairo.npc.ThisApplication
import cn.dairo.npc.page.home.HomePage
import cn.dairo.npc.page.setting.NpcConfigPage
import cn.dairo.npc.repository.NpcRepository
import cn.dairo.npc.ui.theme.MyApplicationTheme
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

@Composable
fun Root(
    vm: RootViewModel = viewModel()
) = MyApplicationTheme {
    ThisApplication.colorScheme = MaterialTheme.colorScheme
//    Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
        val state by vm.state.collectAsState()
        val navController = rememberNavController()
        NavHost(
            navController = navController,
            startDestination = state.firstRoute
        ) {
            composable("welcome") {
                Spacer(modifier = Modifier)
            }

            composable("setting") {
                NpcConfigPage(navController)
            }

            composable("home") {
                HomePage(navController)
            }
        }
//    }
}

class RootViewModel(application: Application) : AndroidViewModel(application) {
    private val _state = MutableStateFlow(
        RootState()
    )
    val state = this._state.asStateFlow()

    init {
        viewModelScope.launch {
            _state.update {
                val repo = NpcRepository(application)
                RootState(firstRoute = if (repo.isSet()) "home" else "setting")
            }
        }
    }
}


data class RootState(
    val firstRoute: String = "welcome"
)