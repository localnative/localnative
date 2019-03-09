package app.localnative.android

object AppState {
    @JvmStatic
    fun makePaginationText(count: Long): String {
        return count.toString()
    }
}