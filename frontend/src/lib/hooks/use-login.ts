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
				const responseText = await response.text();
				let errorMessage = `Login failed (${response.status})`;
				try {
					const errorData = JSON.parse(responseText);
					if (errorData && typeof errorData.message === "string") {
						errorMessage = errorData.message;
					} else {
						errorMessage = responseText || errorMessage;
					}
				} catch {
					errorMessage = responseText || errorMessage;
				}
				throw new Error(errorMessage);
			}

			const dataResponse: LoginResponse = await response.json();
			if (dataResponse.role === "Security") {
				try {
					await fetch(`${config.apiBaseUrl}/auth/logout`, {
						method: "POST",
						credentials: "include"
					});
				} catch (e) {
					console.error("Failed to clear cookie:", e);
				}
				throw new Error("Security personnel are not authorized to access the system.");
			}
			return dataResponse;
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["login"] });
			goto(resolve("/"));
		}
	}));
}
