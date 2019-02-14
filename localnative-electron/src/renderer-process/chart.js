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
var exports = module.exports = {};
const _ = require('underscore');
const neon = require('localnative-neon');
const crossfilter = require('crossfilter2');
const dc = require('dc');

exports.refreshChart = function(days){
  let d3 = require('d3');
  var dateFormatSpecifier = '%Y-%m-%d';
  var dateFormat = d3.timeFormat(dateFormatSpecifier);
  var dateFormatParser = d3.timeParse(dateFormatSpecifier);
  days.forEach(function(d){
    d.dd = dateFormatParser(d.dt)
    d.month = d3.timeMonth(d.dd);
  });
  var ln = crossfilter(days);
  var all = ln.groupAll();

  // Dimension by full date
  var dateDimension = ln.dimension(function (d) {
      return d.dd;
  });

  var months = ln.dimension(function (d) {
      return d.month;
  });
  var monthsGroup = months.group()

  global.lnVolumeChart = dc.barChart('#ln-monthly-volume-chart');

  lnVolumeChart.width(400)
      .height(200)
      .margins({top: 10, right: 50, bottom: 20, left: 40})
      .dimension(months)
      .group(monthsGroup)
      .centerBar(true)
      .gap(1)
      .x(d3.scaleTime().domain([new Date(2008, 0, 1), new Date(2019, 11, 31)]))
      .round(d3.timeMonth.round)
      .alwaysUseRounding(true)
      .xUnits(d3.timeMonths);

  lnVolumeChart.on('filtered', function(chart, filter){
    cmd.filter(filter[0].toUTCString(), filter[1].toUTCString())
  });

  dc.renderAll();
}

const cmd = require('./cmd');
