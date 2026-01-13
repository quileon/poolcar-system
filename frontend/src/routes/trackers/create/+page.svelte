<script lang="ts">
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import { createMutation, useQueryClient } from "@tanstack/svelte-query";
	import { goto } from "$app/navigation";
	import { resolve } from "$app/paths";

	let trackerName = $state("");

	const queryClient = useQueryClient();
	const mutation = createMutation(() => ({
		mutationFn: async (trackerName: string) => {
			const response = await fetch("http://localhost:3000/trackers", {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({ name: trackerName })
			});

			if (!response.ok) {
				throw new Error("Failed to create tracker");
			}

			return response.json();
		},
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: ["trackers"] });
			goto(resolve("/trackers"));
		}
	}));

	function handleSubmit(e: Event) {
		e.preventDefault();
		mutation.mutate(trackerName);
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Tracker</Field.Legend>
				<Field.Description>Tracker will be assigned to each car uniquely.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Tracker Name</Field.Label>
						<Input
							id="name"
							bind:value={trackerName}
							type="text"
							placeholder="Enter tracker name"
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={mutation.isPending}>Submit</Button>
				<Button variant="outline" type="button" disabled={mutation.isPending} href="/trackers"
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
