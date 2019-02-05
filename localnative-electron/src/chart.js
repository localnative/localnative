var exports = module.exports = {};
let _ = require('underscore');

const neon = require('localnative-neon');
crossfilter = require('crossfilter2');
dc = require('dc');

exports.cmdChart = _.debounce(cmdChartImp, 500);

function cmdChartImp(message){
  message.limit = 20000;
  let input = JSON.stringify(message);
  global.resp = JSON.parse(neon.run(input));
  makeChart(resp);
}

function makeChart(resp){
  let d3 = require('d3');
  var dateFormatSpecifier = '%Y-%m-%d %H:%M:%S';
  var dateFormat = d3.timeFormat(dateFormatSpecifier);
  var dateFormatParser = d3.timeParse(dateFormatSpecifier);
  let notes = resp.notes;
  notes.forEach(function(d){
    d.dd = dateFormatParser(d.created_at.substr(0, 19))
    d.month = d3.timeMonth(d.dd);
  });
  var ln = crossfilter(notes);
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

  dc.renderAll();
}
