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

object AppState {
    const val LIMIT = 10
    var offset : Long = 0
    @JvmStatic
    fun makePaginationText(count: Long): String {
        val start = if (count > 0) offset + 1 else 0
        val end : Long = if (offset + LIMIT > count) count else offset + LIMIT

        val p: Long = Math.ceil((0.0 + end) / LIMIT).toLong()
        val z = Math.ceil((count + 0.0) / LIMIT).toLong()
        return "page: $p of $z\nnote: $start-$end of $count"
    }

}