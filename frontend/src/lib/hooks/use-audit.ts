import type { GetAuditResponse } from "$lib/bindings/GetAuditResponse";
import { createQuery } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";

export function useAuditQuery(
	getTrackerId: () => number | null,
	getCarId: () => number | null,
	getDate: () => string | null
) {
	return createQuery<GetAuditResponse>(() => {
		const trackerId = getTrackerId();
		const carId = getCarId();
		const date = getDate();

		return {
			queryKey: ["audit", trackerId, carId, date],
			enabled: trackerId !== null || carId !== null,
			queryFn: async () => {
				const searchParams = new URLSearchParams();
				if (trackerId) {
					searchParams.set("tracker_id", trackerId.toString());
				} else if (carId) {
					searchParams.set("car_id", carId.toString());
				}

				if (date) {
					searchParams.set("date", date);
				}

				const response = await authFetch(`${config.apiBaseUrl}/audit?${searchParams.toString()}`);
				if (!response.ok) throw new Error("Failed to fetch audit");
				return response.json();
			}
		};
	});
}
