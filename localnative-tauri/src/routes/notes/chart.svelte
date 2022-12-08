<script lang="ts">
	import { cmdFilter } from '../cmd';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import * as echarts from 'echarts';

	let mainChartElement: HTMLElement | null;
	let timeSeriesChart: echarts.ECharts;
	let refreshChartUnlistenFn: UnlistenFn | null = null;
	let resetChartZoomUnlistenFn: UnlistenFn | null = null;
	let lastUseDays: Array<{ k: string; v: number; dd: Date; month: Date }> | null = null;

	$: {
		if (mainChartElement) {
			timeSeriesChart = echarts.init(mainChartElement);
		}
	}

	onMount(async () => {
		refreshChartUnlistenFn = await listen<any>('refreshChart', (ev) => {
			console.log(ev.payload.days);
			refreshChart(ev.payload.days);
		});

		resetChartZoomUnlistenFn = await listen<any>('resetChartZoom', (ev) => {
			if (timeSeriesChart) {
				timeSeriesChart.dispatchAction({
					type: 'dataZoom',
					start: 0,
					end: 100
				});
			}
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

		if (resetChartZoomUnlistenFn) {
			resetChartZoomUnlistenFn();
		}
	});

	// const addDays = (dateStr: string, days: number) => {
	// 	var date = new Date(dateStr.valueOf());
	// 	date.setDate(date.getDate() + days);
	// 	return date;
	// };

	// const refreshChart = (days: Array<{ k: string; v: number; dd: Date; month: Date }>) => {
	// 	lastUseDays = days;

	// 	let width =
	// 		document.getElementsByTagName('body')[0].clientWidth -
	// 		(document.getElementById('nav')?.clientWidth ?? 0) -
	// 		(document.getElementById('tags')?.clientWidth ?? 0);

	// 	let lnDayChart = dc.barChart('#ln-day-chart');
	// 	let lnMonthChart = dc.barChart('#ln-month-chart');

	// 	var dateFormatSpecifier = '%Y-%m-%d';
	// 	var dateFormat = d3.timeFormat(dateFormatSpecifier);
	// 	var dateFormatParser = d3.timeParse(dateFormatSpecifier);
	// 	var dtMax = new Date().toISOString().substr(0, 10);
	// 	var dtMin = dtMax;
	// 	if (days.length) {
	// 		dtMin = dtMax = days[0].k;
	// 	}

	// 	days.forEach(function (d) {
	// 		d.dd = dateFormatParser(d.k)!;
	// 		d.month = d3.timeMonth(d.dd ?? undefined);
	// 		dtMin = d.k < dtMin ? d.k : dtMin;
	// 		dtMax = d.k > dtMax ? d.k : dtMax;
	// 	});

	// 	console.log(days);

	// 	var ln = crossfilter(days);
	// 	var all = ln.groupAll();

	// 	// Dimensions
	// 	var daysDimension = ln.dimension(function (d) {
	// 		return d.dd;
	// 	});

	// 	var daysGroup = daysDimension.group().reduceSum(function (d) {
	// 		return d.v;
	// 	});

	// 	var months = ln.dimension(function (d) {
	// 		return d.month;
	// 	});

	// 	var monthsGroup = months.group();

	// 	lnDayChart
	// 		.width(width - 50) // sub margin left and right
	// 		.height(80)
	// 		.transitionDuration(1000)
	// 		.margins({ top: 10, right: 10, bottom: 20, left: 40 })
	// 		.dimension(months)
	// 		.centerBar(true)
	// 		.gap(1)
	// 		.mouseZoomable(true)
	// 		.rangeChart(lnMonthChart)
	// 		.x(d3.scaleTime().domain([addDays(dtMin, -31), addDays(dtMax, 31)]))
	// 		.round(d3.timeMonth.round)
	// 		.xUnits(function (start, stop, step) {
	// 			if (typeof step == 'number' || typeof step == 'undefined') {
	// 				return d3.timeMonths(new Date(start), new Date(stop), step);
	// 			} else {
	// 				return d3.timeMonths(new Date(start), new Date(stop), parseInt(step[0]));
	// 			}
	// 		})
	// 		.alwaysUseRounding(true)
	// 		.elasticY(true)
	// 		.renderHorizontalGridLines(true)
	// 		.brushOn(true)
	// 		.group(daysGroup);

	// 	lnMonthChart
	// 		.width(width - 50) // sub margin left and right
	// 		.height(70)
	// 		.margins({ top: 10, right: 10, bottom: 20, left: 40 })
	// 		.dimension(months)
	// 		.group(monthsGroup)
	// 		.centerBar(true)
	// 		.gap(1)
	// 		.x(d3.scaleTime().domain([addDays(dtMin, -31), addDays(dtMax, 31)]))
	// 		.y(d3.scaleLinear().domain([0, 31]))
	// 		.round(d3.timeMonth.round)
	// 		.alwaysUseRounding(true)
	// 		.brushOn(true)
	// 		.xUnits(function (start, stop, step) {
	// 			if (typeof step == 'number' || typeof step == 'undefined') {
	// 				return d3.timeMonths(new Date(start), new Date(stop), step);
	// 			} else {
	// 				return d3.timeMonths(new Date(start), new Date(stop), parseInt(step[0]));
	// 			}
	// 		})
	// 		.controlsUseVisibility(true)
	// 		.renderHorizontalGridLines(true)
	// 		.on('filtered', (chart, filter) => {
	// 			if (filter) {
	// 				// lnDayChart.focus(filter);
	// 				globalThis.AppState.clearOffset();

	// 				let range_from = filter[0].toISOString().substr(0, 10);
	// 				let range_to = filter[1].toISOString().substr(0, 10);
	// 				globalThis.AppState.range = [range_from, range_to];

	// 				cmdFilter('', range_from, range_to);
	// 			}
	// 		})
	// 		.yAxis()
	// 		.tickValues([0, 5, 10, 15, 20, 25, 30]);

	// 	dc.renderAll();
	// };

	const refreshChart = (days: Array<{ k: string; v: number }>) => {
		var option;

		const dateList = days.map(function (item) {
			return item.k;
		});

		const valueList = days.map(function (item) {
			return item.v;
		});

		option = {
			grid: {
				top: '8px',
				right: '76px',
				bottom: '80px',
				left: '70px'
			},
			dataZoom: [
				{
					type: 'slider'
				},
				{
					type: 'inside'
				}
			],
			tooltip: {
				trigger: 'axis'
			},
			xAxis: {
				data: dateList
			},
			yAxis: {
				// splitLine: {
				// 	lineStyle: {
				// 		color: '#a6adba'
				// 	}
				// }
			},
			series: {
				type: 'bar',
				showSymbol: false,
				data: valueList
			}
		};
		timeSeriesChart.on('datazoom', function (params) {
			var option: any = timeSeriesChart.getOption();
			if (option.dataZoom) {
				const start = new Date(days[option.dataZoom[0].startValue].k)
					.toISOString()
					.substring(0, 10);

				const end = new Date(days[option.dataZoom[0].endValue].k).toISOString().substring(0, 10);

				globalThis.AppState.clearOffset();
				globalThis.AppState.range = [Number(start), Number(end)];

				cmdFilter('', start, end);
			}
		});
		option && timeSeriesChart.setOption(option);
	};
</script>

<slot>
	<div id="main" class="w-full h-40">
		<div class="w-full h-full" bind:this={mainChartElement} />
	</div>
</slot>

<style>
</style>
