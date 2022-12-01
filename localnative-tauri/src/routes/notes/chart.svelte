<script lang="ts">
	import * as d3 from 'd3';
	import * as dc from 'dc';
	import crossfilter from 'crossfilter2';
	import { cmdFilter } from '../cmd';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';

	let refreshChartUnlistenFn: UnlistenFn | null = null;
	let lnDayChart = dc.barChart('#ln-day-chart');
	let lnMonthChart = dc.barChart('#ln-month-chart');

	onMount(async () => {
		refreshChartUnlistenFn = await listen<any>('refreshChart', (ev) => {
			console.log(ev.payload.days);
			refreshChart(ev.payload.days);
		});
	});

	onDestroy(() => {
		if (refreshChartUnlistenFn) {
			refreshChartUnlistenFn();
		}
	});

	const addDays = (dateStr: string, days: number) => {
		var date = new Date(dateStr.valueOf());
		date.setDate(date.getDate() + days);
		return date;
	};

	const refreshChart = (days: Array<{ k: string; v: number; dd: Date; month: Date }>) => {
		var dateFormatSpecifier = '%Y-%m-%d';
		var dateFormat = d3.timeFormat(dateFormatSpecifier);
		var dateFormatParser = d3.timeParse(dateFormatSpecifier);
		var dtMax = new Date().toISOString().substr(0, 10);
		var dtMin = dtMax;
		if (days.length) {
			dtMin = dtMax = days[0].k;
		}

		days.forEach(function (d) {
			d.dd = dateFormatParser(d.k)!;
			d.month = d3.timeMonth(d.dd ?? undefined);
			dtMin = d.k < dtMin ? d.k : dtMin;
			dtMax = d.k > dtMax ? d.k : dtMax;
		});

		console.log(days);

		var ln = crossfilter(days);
		var all = ln.groupAll();

		// Dimensions
		var daysDimension = ln.dimension(function (d) {
			return d.dd;
		});

		var daysGroup = daysDimension.group().reduceSum(function (d) {
			return d.v;
		});

		var months = ln.dimension(function (d) {
			return d.month;
		});

		var monthsGroup = months.group();

		lnDayChart
			.width(800)
			.height(150)
			.transitionDuration(1000)
			.margins({ top: 10, right: 50, bottom: 20, left: 40 })
			.dimension(months)
			.centerBar(true)
			.gap(1)
			.mouseZoomable(true)
			.rangeChart(lnMonthChart)
			.x(d3.scaleTime().domain([addDays(dtMin, -31), addDays(dtMax, 31)]))
			.round(d3.timeMonth.round)

			.xUnits(function (start, stop, step) {
				if (typeof step == 'number' || typeof step == 'undefined') {
					return d3.timeMonths(new Date(start), new Date(stop), step);
				} else {
					return d3.timeMonths(new Date(start), new Date(stop), parseInt(step[0]));
				}
			})
			.elasticY(true)
			.renderHorizontalGridLines(true)
			.brushOn(false)
			.group(daysGroup);

		lnMonthChart
			.width(800)
			.height(100)
			.margins({ top: 10, right: 50, bottom: 20, left: 40 })
			.dimension(months)
			.group(monthsGroup)
			.centerBar(true)
			.gap(1)
			.x(d3.scaleTime().domain([addDays(dtMin, -31), addDays(dtMax, 31)]))
			.y(d3.scaleLinear().domain([0, 31]))
			.round(d3.timeMonth.round)
			.alwaysUseRounding(true)
			.xUnits(function (start, stop, step) {
				if (typeof step == 'number' || typeof step == 'undefined') {
					return d3.timeMonths(new Date(start), new Date(stop), step);
				} else {
					return d3.timeMonths(new Date(start), new Date(stop), parseInt(step[0]));
				}
			})
			.renderHorizontalGridLines(true)
			.yAxis()
			.tickValues([0, 5, 10, 15, 20, 25, 30]);

		lnMonthChart.on('filtered', function (chart, filter) {
			if (filter) {
				lnDayChart.focus(filter);
				globalThis.AppState.clearOffset();

				let range_from = filter[0].toISOString().substr(0, 10);
				let range_to = filter[1].toISOString().substr(0, 10);
				globalThis.AppState.range = [range_from, range_to];

				cmdFilter('', range_from, range_to);
			}
		});

		dc.renderAll();
	};
</script>

<slot>
	<div class="container">
		<div id="ln-day-chart">
			<!-- <strong id="title-charts">Timeseries Charts</strong> -->
			<!-- <span class="reset" style="display: none;">range: <span class="filter"></span></span> -->
			<!-- <a class="reset" href="javascript:lnDayChart.filterAll();lnMonthChart.filterAll();dc.redrawAll();" style="display: none;">reset</a> -->
			<!-- <div class="clearfix" /> -->
		</div>
		<div id="ln-month-chart">
			<p>
				<span id="select-range-prompt">select a time range to zoom in</span>:
				<span class="reset" style="display: none;"> <span class="filter" /></span>
			</p>
		</div>
	</div>
</slot>

<style>
	.container {
		width: 800px;
		height: 600px;
		/* overflow-y: auto; */
	}

	#ln-monthly-volume-chart g.y {
		/* display: none; */
	}
</style>
