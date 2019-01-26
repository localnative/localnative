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

var argv = process.argv.slice(2)

if (argv[0] == 'ssb-publish'){
  let pubkeys = argv[1]
  require('./ssb/publish')(pubkeys)
}

if (argv[0] == 'ssbify'){
  let html = argv[1]
  let title = argv[2]
  let url = argv[3]
  require('./ssb/ssbify')(html, title, url)
}

if (argv[0] == 'ssb-tail'){
  let id = argv[1]
  let gt = argv[2]
  require('./ssb/tail')(id, gt)
}

if (argv[0] == 'ssb-whoami'){
  require('./ssb/whoami')()
}
