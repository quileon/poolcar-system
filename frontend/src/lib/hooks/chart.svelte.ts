import type { MqttPayloadWithTrackerCar } from "$lib/bindings/MqttPayloadWithTrackerCar";

export type ChartDataPoint = {
	time: Date;
	[trackerId: string]: number | Date;
};

export class LatencyChart {
	#data: ChartDataPoint[] = $state([]);

	get data() {
		return this.#data;
	}

	addAuditData(trackersData: Record<string, MqttPayloadWithTrackerCar | null>) {
		const dataPoint: ChartDataPoint = {
			// eslint-disable-next-line svelte/prefer-svelte-reactivity
			time: new Date()
		};

		for (const [trackerId, payload] of Object.entries(trackersData)) {
			if (payload) {
				dataPoint[trackerId] = payload.connection.interval;
			}
		}

		// 1. Build a new array reference
		const newData = [...this.#data, dataPoint];

		if (newData.length > 60) {
			newData.shift();
		}

		// 2. Reassign to trigger a single, clean reactivity update
		this.#data = newData;
	}

	clear() {
		this.#data = [];
	}
}
