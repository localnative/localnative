<script lang="ts">
	import { cmdFilter } from '../cmd';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import * as echarts from 'echarts';

	/** One bar per day, as the core's `days` facet sends it. */
	type Day = { date: string; count: number };

	let mainChartElement: HTMLElement | null = null;
	let timeSeriesChart: echarts.ECharts | null = null;
	let refreshChartUnlistenFn: UnlistenFn | null = null;
	let resetChartZoomUnlistenFn: UnlistenFn | null = null;
	// The days behind the drawn bars; the zoom window indexes into them.
	let drawnDays: Day[] | null = null;

	const onResize = () => timeSeriesChart?.resize();

	onMount(async () => {
		if (mainChartElement) {
			timeSeriesChart = echarts.init(mainChartElement);
			// Registered once: registering on every redraw stacked up one
			// handler per redraw, each firing its own filter for one zoom.
			timeSeriesChart.on('datazoom', onDataZoom);
		}

		refreshChartUnlistenFn = await listen<{ days: Day[] }>('refreshChart', (ev) => {
			refreshChart(ev.payload.days);
		});

		resetChartZoomUnlistenFn = await listen('resetChartZoom', () => {
			// Silent: a programmatic reset must not fire a filter of its own.
			timeSeriesChart?.dispatchAction({ type: 'dataZoom', start: 0, end: 100 }, { silent: true });
		});

		window.addEventListener('resize', onResize);
	});

	onDestroy(() => {
		refreshChartUnlistenFn?.();
		resetChartZoomUnlistenFn?.();
		window.removeEventListener('resize', onResize);
		timeSeriesChart?.dispose();
		timeSeriesChart = null;
	});

	/** The zoom window selects an inclusive day range (`YYYY-MM-DD`). */
	const onDataZoom = () => {
		if (!timeSeriesChart || !drawnDays || drawnDays.length === 0) return;
		// After a zoom echarts writes the window back as category indices.
		const zoom = (timeSeriesChart.getOption() as any).dataZoom?.[0];
		if (!zoom) return;
		const last = drawnDays.length - 1;
		const first = drawnDays[Math.min(last, Math.max(0, zoom.startValue ?? 0))];
		const end = drawnDays[Math.min(last, Math.max(0, zoom.endValue ?? last))];
		if (!first || !end) return;

		globalThis.AppState.clearOffset();
		globalThis.AppState.range = [first.date, end.date];
		cmdFilter(globalThis.AppState.query, first.date, end.date);
	};

	const sameDays = (a: Day[], b: Day[]) =>
		a.length === b.length && a.every((day, i) => day.date === b[i].date && day.count === b[i].count);

	const refreshChart = (days: Day[]) => {
		if (!timeSeriesChart) return;
		// The histogram always spans every day, so the reply to a range filter
		// carries the bars already drawn: leave the zoom window alone then. A
		// new query draws new bars and starts from the full range.
		if (drawnDays && sameDays(days, drawnDays)) return;
		drawnDays = days;

		timeSeriesChart.setOption({
			grid: {
				top: '8px',
				right: '76px',
				bottom: '80px',
				left: '70px'
			},
			dataZoom: [{ type: 'slider', start: 0, end: 100 }, { type: 'inside', start: 0, end: 100 }],
			tooltip: {
				trigger: 'axis'
			},
			xAxis: {
				type: 'category',
				data: days.map((day) => day.date)
			},
			yAxis: {
				type: 'value',
				minInterval: 1
			},
			series: [
				{
					type: 'bar',
					data: days.map((day) => day.count)
				}
			]
		});
	};
</script>

<slot>
	<div id="main" class="w-full h-40">
		<div class="w-full h-full" bind:this={mainChartElement}></div>
	</div>
</slot>
