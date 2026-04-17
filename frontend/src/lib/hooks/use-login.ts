import type { LoginResponse } from "$lib/bindings/LoginResponse";
import { createMutation, useQueryClient } from "@tanstack/svelte-query";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

export function useLoginMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { username: string; password: string }) => {
			const response = await fetch(`${config.apiBaseUrl}/auth/login`, {
				method: "POST",
				credentials: "include",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					username: data.username,
					password: data.password
				})
			});
			if (!response.ok) {
				const errorMessage = await response.text();
				throw new Error(errorMessage || `Login failed (${response.status})`);
			}

			const dataResponse: LoginResponse = await response.json();
			return dataResponse;
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["login"] });
			goto(resolve("/"));
		}
	}));
}
