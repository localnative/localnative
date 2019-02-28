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
global.dc = require('dc');

Date.prototype.addDays = function(days) {
    var date = new Date(this.valueOf());
    date.setDate(date.getDate() + days);
    return date;
}

global.lnDayChart = dc.barChart('#ln-day-chart');
global.lnMonthChart = dc.barChart('#ln-month-chart');

exports.refreshChart = function(days){
  let d3 = require('d3');
  var dateFormatSpecifier = '%Y-%m-%d';
  var dateFormat = d3.timeFormat(dateFormatSpecifier);
  var dateFormatParser = d3.timeParse(dateFormatSpecifier);
  var dtMin = dtMax = (new Date()).toISOString().substr(0,10);
  if (days.length){
    dtMin = dtMax = days[0].k;
  }
  days.forEach(function(d){
    d.dd = dateFormatParser(d.k);
    d.month = d3.timeMonth(d.dd);
    dtMin = d.k < dtMin ? d.k : dtMin;
    dtMax = d.k > dtMax ? d.k : dtMax;
  });
  var ln = crossfilter(days);
  var all = ln.groupAll();

  // Dimensions
  var daysDimension = ln.dimension(function (d) {
    return d.dd;
  });
  var daysGroup = daysDimension.group().reduceSum(function(d){
    return d.v;
  });

  var months = ln.dimension(function (d) {
      return d.month;
  });
  var monthsGroup = months.group()


  lnDayChart
      .width(800)
      .height(150)
      .transitionDuration(1000)
      .margins({top: 10, right: 50, bottom: 20, left: 40})
      // .dimension(daysDimension)
      .dimension(months)
      .centerBar(true)
      .gap(1)
      .mouseZoomable(true)
      .rangeChart(lnMonthChart)
      .x(d3.scaleTime().domain([(new Date(dtMin)).addDays(-31), (new Date(dtMax)).addDays(31)]))
      .round(d3.timeMonth.round)
      // .alwaysUseRounding(true)
      .xUnits(d3.timeMonths)
      .elasticY(true)
      .renderHorizontalGridLines(true)
      .brushOn(false)
      .group(daysGroup)

  lnMonthChart.width(800)
      .height(100)
      .margins({top: 10, right: 50, bottom: 20, left: 40})
      .dimension(months)
      .group(monthsGroup)
      .centerBar(true)
      .gap(1)
      .x(d3.scaleTime().domain([(new Date(dtMin)).addDays(-31), (new Date(dtMax)).addDays(31)]))
      .y(d3.scaleLinear().domain([0, 31]))
      .round(d3.timeMonth.round)
      .alwaysUseRounding(true)
      .xUnits(d3.timeMonths)
      .renderHorizontalGridLines(true)
      .yAxis().tickValues([0, 5, 10, 15, 20, 25, 30])

  lnMonthChart.on('filtered', function(chart, filter){
    if (filter){
      lnDayChart.focus(filter)
      cmd.setOffset(0);
      cmd.cmdFilter(filter[0].toISOString().substr(0,10)
        , filter[1].toISOString().substr(0,10)
      );
    }
  });

  dc.renderAll();
}

const cmd = require('./cmd');
