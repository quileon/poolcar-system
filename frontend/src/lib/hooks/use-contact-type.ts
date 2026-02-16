import type { ContactTypeWithCount } from "$lib/bindings/ContactTypeWithCount";
import type { GetContactTypesResponse } from "$lib/bindings/GetContactTypesResponse";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

export function useContactTypesQuery() {
	return createQuery<GetContactTypesResponse[]>(() => ({
		queryKey: ["contact-types"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/contacts/types`);
			if (!response.ok) throw new Error("Failed to fetch contact types");
			return response.json();
		}
	}));
}

export function useContactTypeQuery(getContactTypeId: () => number) {
	return createQuery<ContactTypeWithCount>(() => ({
		queryKey: ["contact-type", getContactTypeId()],
		queryFn: async () => {
			const contactTypeId = getContactTypeId();
			const response = await authFetch(`${config.apiBaseUrl}/contacts/types/${contactTypeId}`);
			if (!response.ok) throw new Error("Failed to fetch contact type");
			return response.json();
		}
	}));
}

export function useCreateContactTypeMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const response = await authFetch(`${config.apiBaseUrl}/contacts/types`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to create contact type");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/contact-types"));
			await queryClient.invalidateQueries({ queryKey: ["contact-types"] });
		}
	}));
}

export function useEditContactTypeMutation(getContactTypeId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: { name: string }) => {
			const contactTypeId = getContactTypeId();
			const response = await authFetch(`${config.apiBaseUrl}/contacts/types/${contactTypeId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name
				})
			});
			if (!response.ok) throw new Error("Failed to modify contact type");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/contact-types"));
			await queryClient.invalidateQueries({ queryKey: ["contact-types"] });
		}
	}));
}

export function useDeleteContactTypeMutation(getContactTypeId: () => number) {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async () => {
			const contactTypeId = getContactTypeId();
			const response = await authFetch(`${config.apiBaseUrl}/contacts/types/${contactTypeId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete contact type");
			return response.json();
		},
		onSuccess: async () => {
			await goto(resolve("/contact-types"));
			await queryClient.invalidateQueries({ queryKey: ["contact-types"] });
		}
	}));
}
