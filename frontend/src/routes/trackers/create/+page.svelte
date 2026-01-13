<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
</script>

<div class="mx-auto w-full max-w-md">
	<form>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Tracker</Field.Legend>
				<Field.Description>Tracker will be assigned to each car uniquely.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Tracker Name</Field.Label>
						<Input id="name" name="name" type="text" placeholder="Enter tracker name" required />
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit">Submit</Button>
				<Button variant="outline" type="button">Cancel</Button>
			</Field.Field>
		</Field.Group>
	</form>
</div>
