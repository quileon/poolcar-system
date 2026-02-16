import type { GetMqttPayloadHistory } from "$lib/bindings/GetMqttPayloadHistory";
import { createQuery } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";

export function useMqttPayloadHistoriesQuery() {
	return createQuery<GetMqttPayloadHistory[]>(() => ({
		queryKey: ["mqtt-payload-histories"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/live`);
			if (!response.ok) throw new Error("Failed to fetch MQTT payload histories");
			return response.json();
		}
	}));
}
