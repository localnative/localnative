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

import android.content.Intent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.QrCode
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    viewModel: MainViewModel = viewModel(),
    onQRScanClick: () -> Unit
) {
    val uiState by viewModel.uiState.collectAsState()
    var searchText by remember { mutableStateOf("") }
    var isSearchActive by remember { mutableStateOf(false) }
    var showDeleteDialog by remember { mutableStateOf<NoteItem?>(null) }

    Scaffold(
        topBar = {
            if (isSearchActive) {
                SearchBar(
                    query = searchText,
                    onQueryChange = { searchText = it },
                    onSearch = {
                        viewModel.search(it)
                        isSearchActive = false
                    },
                    active = true,
                    onActiveChange = { isSearchActive = it },
                    placeholder = { Text("Search notes...") },
                    leadingIcon = { Icon(Icons.Default.Search, contentDescription = "Search") },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    // Search suggestions could go here
                }
            } else {
                TopAppBar(
                    title = { Text("Local Native") },
                    actions = {
                        IconButton(onClick = { isSearchActive = true }) {
                            Icon(Icons.Default.Search, contentDescription = "Search")
                        }
                        IconButton(onClick = onQRScanClick) {
                            Icon(Icons.Default.QrCode, contentDescription = "Sync")
                        }
                    }
                )
            }
        },
        bottomBar = {
            PaginationBar(
                paginationText = uiState.paginationText,
                hasPrevious = uiState.hasPreviousPage,
                hasNext = uiState.hasNextPage,
                onPreviousClick = { viewModel.previousPage() },
                onNextClick = { viewModel.nextPage() }
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            when {
                uiState.isLoading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                uiState.error != null -> {
                    Column(
                        modifier = Modifier
                            .align(Alignment.Center)
                            .padding(16.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = uiState.error ?: "Unknown error",
                            color = MaterialTheme.colorScheme.error
                        )
                        Button(
                            onClick = { viewModel.clearError() },
                            modifier = Modifier.padding(top = 8.dp)
                        ) {
                            Text("Dismiss")
                        }
                    }
                }
                uiState.notes.isEmpty() -> {
                    Text(
                        text = "No notes found",
                        modifier = Modifier.align(Alignment.Center),
                        style = MaterialTheme.typography.bodyLarge
                    )
                }
                else -> {
                    NoteList(
                        notes = uiState.notes,
                        onDeleteClick = { showDeleteDialog = it },
                        onQRCodeClick = { note ->
                            viewModel.setCurrentUrl(note.url)
                        },
                        onTagClick = { tag ->
                            searchText = tag
                            viewModel.search(tag)
                        }
                    )
                }
            }
        }
    }

    // Delete confirmation dialog
    showDeleteDialog?.let { note ->
        AlertDialog(
            onDismissRequest = { showDeleteDialog = null },
            title = { Text("Delete Note") },
            text = { Text("Are you sure you want to delete this note?") },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.deleteNote(note.rowid)
                        showDeleteDialog = null
                    }
                ) {
                    Text("Delete", color = Color.Red)
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = null }) {
                    Text("Cancel")
                }
            }
        )
    }
}

@Composable
fun NoteList(
    notes: List<NoteItem>,
    onDeleteClick: (NoteItem) -> Unit,
    onQRCodeClick: (NoteItem) -> Unit,
    onTagClick: (String) -> Unit
) {
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        items(notes, key = { it.rowid }) { note ->
            NoteCard(
                note = note,
                onDeleteClick = { onDeleteClick(note) },
                onQRCodeClick = { onQRCodeClick(note) },
                onTagClick = onTagClick
            )
        }
    }
}

@Composable
fun NoteCard(
    note: NoteItem,
    onDeleteClick: () -> Unit,
    onQRCodeClick: () -> Unit,
    onTagClick: (String) -> Unit
) {
    val context = LocalContext.current

    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp)
        ) {
            // Tags and action buttons
            LazyRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                // Delete button
                item {
                    Button(
                        onClick = onDeleteClick,
                        colors = ButtonDefaults.buttonColors(
                            containerColor = Color.Transparent,
                            contentColor = Color.Red
                        ),
                        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)
                    ) {
                        Text("X")
                    }
                }

                // QR Code button
                item {
                    Button(
                        onClick = {
                            onQRCodeClick()
                            val intent = Intent(context, QRCodeActivity::class.java)
                            intent.putExtra(NoteContent.NOTE_ITEM, note.toSerializable())
                            context.startActivity(intent)
                        },
                        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)
                    ) {
                        Text("QR")
                    }
                }

                // Tag buttons
                items(note.getTagList()) { tag ->
                    Button(
                        onClick = { onTagClick(tag) },
                        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)
                    ) {
                        Text(tag)
                    }
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            // Note content
            Text(
                text = "${note.createdAt.take(19)} uuid ${note.uuid4.take(5)}.. rowid ${note.rowid}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            if (note.title.isNotEmpty()) {
                Text(
                    text = note.title,
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.padding(top = 4.dp),
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }

            if (note.description.isNotEmpty()) {
                Text(
                    text = note.description,
                    style = MaterialTheme.typography.bodyMedium,
                    modifier = Modifier.padding(top = 4.dp),
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis
                )
            }

            if (note.url.isNotEmpty()) {
                Text(
                    text = note.url,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.padding(top = 4.dp),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}

@Composable
fun PaginationBar(
    paginationText: String,
    hasPrevious: Boolean,
    hasNext: Boolean,
    onPreviousClick: () -> Unit,
    onNextClick: () -> Unit
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        tonalElevation = 3.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Button(
                onClick = onPreviousClick,
                enabled = hasPrevious
            ) {
                Text("Previous")
            }

            Text(
                text = paginationText,
                style = MaterialTheme.typography.bodyMedium
            )

            Button(
                onClick = onNextClick,
                enabled = hasNext
            ) {
                Text("Next")
            }
        }
    }
}

// Extension function to convert NoteItem to serializable NoteContent.NoteItem
fun NoteItem.toSerializable(): NoteContent.NoteItem {
    return NoteContent.NoteItem(
        rowid = this.rowid,
        uuid4 = this.uuid4,
        title = this.title,
        url = this.url,
        tags = this.tags,
        description = this.description,
        comments = this.comments,
        annotations = this.annotations,
        created_at = this.createdAt,
        is_public = this.isPublic
    )
}
