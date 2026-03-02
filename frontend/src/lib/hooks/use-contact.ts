import type { ContactDetails } from "$lib/bindings/ContactDetails";
import type { GetContactsResponse } from "$lib/bindings/GetContactsResponse";
import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { authFetch } from "./auth.svelte";
import { config } from "$lib/config";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

export function useContactsQuery() {
	return createQuery<GetContactsResponse>(() => ({
		queryKey: ["contacts"],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/contacts`);
			if (!response.ok) throw new Error("Failed to fetch contacts");
			return response.json();
		}
	}));
}

export function useContactQuery(getContactId: () => number) {
	const contactId = getContactId();

	return createQuery<ContactDetails>(() => ({
		queryKey: ["contact", getContactId()],
		queryFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/contacts/${contactId}`);
			if (!response.ok) throw new Error("Failed to fetch contact");
			return response.json();
		}
	}));
}

export function useCreateContactMutation() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: async (data: {
			name: string;
			latitude: number;
			longitude: number;
			contactTypeId: number;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/contacts`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name,
					latitude: data.latitude,
					longitude: data.longitude,
					contact_type_id: data.contactTypeId
				})
			});
			if (!response.ok) throw new Error("Failed to create contact");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["contacts"] });
			await goto(resolve("/contacts"));
		}
	}));
}

export function useEditContactMutation(getContactId: () => number) {
	const queryClient = useQueryClient();
	const contactId = getContactId();

	return createMutation(() => ({
		mutationFn: async (data: {
			name: string;
			latitude: number;
			longitude: number;
			contactTypeId: number;
		}) => {
			const response = await authFetch(`${config.apiBaseUrl}/contacts/${contactId}`, {
				method: "PUT",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.name,
					latitude: data.latitude,
					longitude: data.longitude,
					contact_type_id: data.contactTypeId
				})
			});
			if (!response.ok) throw new Error("Failed to modify contact");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["contacts"] });
			await queryClient.invalidateQueries({ queryKey: ["contact", contactId] });
			await goto(resolve("/contacts"));
		}
	}));
}

export function useDeleteContactMutation(getContactId: () => number) {
	const queryClient = useQueryClient();
	const contactId = getContactId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/contacts/${contactId}`, {
				method: "DELETE"
			});
			if (!response.ok) throw new Error("Failed to delete contact");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["contacts"] });
			await queryClient.invalidateQueries({ queryKey: ["contact", contactId] });
			await goto(resolve("/contacts"));
		}
	}));
}

export function useRestoreContactmutation(getContactId: () => number) {
	const queryClient = useQueryClient();
	const contactId = getContactId();

	return createMutation(() => ({
		mutationFn: async () => {
			const response = await authFetch(`${config.apiBaseUrl}/contacts/${contactId}/restore`, {
				method: "PUT"
			});
			if (!response.ok) throw new Error("Failed to restore contact");
			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["contacts"] });
			await queryClient.invalidateQueries({ queryKey: ["contact", contactId] });
			await goto(resolve("/contacts"));
		}
	}));
}
