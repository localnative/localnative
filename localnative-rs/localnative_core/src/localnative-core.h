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
#ifndef LOCALNATIVE_CORE_H
#define LOCALNATIVE_CORE_H

#ifdef __cplusplus
extern "C" {
#endif

/* Run one JSON command; the reply is JSON. Every failure is
 * {"error": <message>, "code": <stable code>}. Free the reply with
 * localnative_free. */
char* localnative_run(const char* json_input);

/* Point the core at a database file (copied; call before localnative_run).
 * Without it the platform default (~/LocalNative/localnative.sqlite3) is
 * used. */
void localnative_set_db_path(const char* path);

/* Free a string returned by localnative_run. */
void localnative_free(char* s);

#ifdef __cplusplus
}
#endif

#endif /* LOCALNATIVE_CORE_H */
