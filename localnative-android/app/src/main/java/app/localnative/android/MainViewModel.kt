/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
package app.localnative.android

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import org.json.JSONObject

data class NoteItem(
    val rowid: Int,
    val uuid4: String,
    val title: String,
    val url: String,
    val tags: String,
    val description: String,
    val comments: String,
    val annotations: String,
    val createdAt: String,
    val isPublic: Boolean
) {
    fun getTagList(): List<String> {
        return tags.split(",").filter { it.isNotEmpty() }
    }
}

data class MainUiState(
    val notes: List<NoteItem> = emptyList(),
    val query: String = "",
    val offset: Long = 0,
    val count: Long = 0,
    val limit: Int = 10,
    val isLoading: Boolean = false,
    val error: String? = null,
    val currentUrl: String = ""
) {
    val paginationText: String
        get() {
            val start = if (count > 0) offset + 1 else 0
            val end = if (offset + limit > count) count else offset + limit
            return "$start-$end / $count"
        }

    val hasNextPage: Boolean
        get() = offset + limit < count

    val hasPreviousPage: Boolean
        get() = offset > 0
}

class MainViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(MainUiState())
    val uiState: StateFlow<MainUiState> = _uiState.asStateFlow()

    init {
        // Load initial notes
        search("")
    }

    fun search(query: String) {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, query = query, offset = 0) }
            performSearch(query, 0)
        }
    }

    fun nextPage() {
        viewModelScope.launch {
            val currentState = _uiState.value
            if (currentState.hasNextPage) {
                val newOffset = currentState.offset + currentState.limit
                _uiState.update { it.copy(offset = newOffset, isLoading = true) }
                performSearch(currentState.query, newOffset)
            }
        }
    }

    fun previousPage() {
        viewModelScope.launch {
            val currentState = _uiState.value
            if (currentState.hasPreviousPage) {
                val newOffset = maxOf(0, currentState.offset - currentState.limit)
                _uiState.update { it.copy(offset = newOffset, isLoading = true) }
                performSearch(currentState.query, newOffset)
            }
        }
    }

    fun deleteNote(rowid: Int) {
        viewModelScope.launch {
            val currentState = _uiState.value
            val cmd = """{"action": "delete", "query": "${currentState.query}", "rowid": $rowid, "limit": ${currentState.limit}, "offset": ${currentState.offset}}"""
            Log.d("deleteNote", cmd)

            try {
                val response = RustBridge.run(cmd)
                Log.d("deleteNoteResponse", response)
                // Refresh the current page
                performSearch(currentState.query, currentState.offset)
            } catch (e: Exception) {
                Log.e("deleteNote", "Error deleting note", e)
                _uiState.update { it.copy(error = "Failed to delete note: ${e.message}") }
            }
        }
    }

    fun setCurrentUrl(url: String) {
        _uiState.update { it.copy(currentUrl = url) }
    }

    fun clearError() {
        _uiState.update { it.copy(error = null) }
    }

    private suspend fun performSearch(query: String, offset: Long) {
        try {
            val currentState = _uiState.value
            val cmd = """{"action": "search", "query": "$query", "limit": ${currentState.limit}, "offset": $offset}"""
            Log.d("performSearch", cmd)

            val response = RustBridge.run(cmd)
            Log.d("performSearchResponse", response)

            val jsonObject = JSONObject(response)
            val count = jsonObject.getLong("count")
            val notesArray = jsonObject.getJSONArray("notes")

            val notes = mutableListOf<NoteItem>()
            for (i in 0 until notesArray.length()) {
                val noteJson = notesArray.getJSONObject(i)
                notes.add(
                    NoteItem(
                        rowid = noteJson.getInt("rowid"),
                        uuid4 = noteJson.getString("uuid4"),
                        title = noteJson.getString("title"),
                        url = noteJson.getString("url"),
                        tags = noteJson.getString("tags"),
                        description = noteJson.getString("description"),
                        comments = noteJson.getString("comments"),
                        annotations = noteJson.getString("annotations"),
                        createdAt = noteJson.getString("created_at"),
                        isPublic = noteJson.getBoolean("is_public")
                    )
                )
            }

            _uiState.update {
                it.copy(
                    notes = notes,
                    count = count,
                    isLoading = false,
                    error = null
                )
            }
        } catch (e: Exception) {
            Log.e("performSearch", "Error performing search", e)
            _uiState.update {
                it.copy(
                    isLoading = false,
                    error = "Failed to load notes: ${e.message}"
                )
            }
        }
    }
}
