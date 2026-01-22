<script lang="ts">
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { goto } from "$app/navigation";
	import { resolve } from "$app/paths";
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import { createMutation, useQueryClient } from "@tanstack/svelte-query";
	import { config } from "$lib/config";

	let carTypeName = $state("");

	const queryClient = useQueryClient();
	const mutation = createMutation(() => ({
		mutationFn: async (carTypeName: string) => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({ name: carTypeName })
			});

			if (!response.ok) {
				throw new Error("Failed to create car type");
			}

			return response.json();
		},

		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["car-types"] });
			goto(resolve("/car-types"));
		}
	}));

	function handleSubmit(e: Event) {
		e.preventDefault();
		mutation.mutate(carTypeName);
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Car Type</Field.Legend>
				<Field.Description>Car Type will be used to categorize cars.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Car Type Name</Field.Label>
						<Input
							id="name"
							bind:value={carTypeName}
							name="name"
							type="text"
							placeholder="Enter car type name"
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={mutation.isPending}>Submit</Button>
				<Button variant="outline" type="button" disabled={mutation.isPending} href="/cars"
					>Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if mutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{mutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
