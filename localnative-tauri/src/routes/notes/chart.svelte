<script lang="ts">
	import * as d3 from 'd3';
	import * as dc from 'dc';
	import crossfilter from 'crossfilter2';
	import { cmdFilter } from '../cmd';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import 'dc/src/compat/d3v6';

	let chartContainer: Element;
	let refreshChartUnlistenFn: UnlistenFn | null = null;
	let lastUseDays: Array<{ k: string; v: number; dd: Date; month: Date }> | null = null;

	onMount(async () => {
		refreshChartUnlistenFn = await listen<any>('refreshChart', (ev) => {
			console.log(ev.payload.days);
			refreshChart(ev.payload.days);
		});

		window.addEventListener('resize', () => {
			if (lastUseDays != null) {
				refreshChart(lastUseDays);
			}
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
		lastUseDays = days;

		let width =
			document.getElementsByTagName('body')[0].clientWidth -
			(document.getElementById('nav')?.clientWidth ?? 0) -
			(document.getElementById('tags')?.clientWidth ?? 0);

		let lnDayChart = dc.barChart('#ln-day-chart');
		let lnMonthChart = dc.barChart('#ln-month-chart');

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
			.width(width - 50) // sub margin left and right
			.height(150)
			.transitionDuration(1000)
			.margins({ top: 10, right: 10, bottom: 20, left: 40 })
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
			.alwaysUseRounding(true)
			.elasticY(true)
			.renderHorizontalGridLines(true)
			.brushOn(false)
			.group(daysGroup);

		lnMonthChart
			.width(width - 50) // sub margin left and right
			.height(100)
			.margins({ top: 10, right: 10, bottom: 20, left: 40 })
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
			.on('filtered', (chart, filter) => {
				if (filter) {
					lnDayChart.focus(filter);
					globalThis.AppState.clearOffset();

					let range_from = filter[0].toISOString().substr(0, 10);
					let range_to = filter[1].toISOString().substr(0, 10);
					globalThis.AppState.range = [range_from, range_to];

					cmdFilter('', range_from, range_to);
				}
			})
			.yAxis()
			.tickValues([0, 5, 10, 15, 20, 25, 30]);

		dc.renderAll();
	};
</script>

<slot>
	<div class="w-full" bind:this={chartContainer}>
		<div id="ln-day-chart">
			<strong id="title-charts">Timeseries Charts</strong>
			<!-- <span class="reset" style="display: none;">range: <span class="filter" /></span>
			<a
				class="reset"
				href="javascript:lnDayChart.filterAll();lnMonthChart.filterAll();dc.redrawAll();"
				style="display: none;">reset</a
			> -->
		</div>

		<div id="ln-month-chart">
			<div class="flex flex-row">
				<div id="select-range-prompt">
					<strong>Select a time range to zoom in:&nbsp;</strong>
				</div>

				<div class="reset" style="display: none;">
					<span class="filter" />
				</div>
			</div>
		</div>
	</div>
</slot>

<style>
</style>
