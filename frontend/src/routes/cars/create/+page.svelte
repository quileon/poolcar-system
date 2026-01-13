<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
</script>

<div class="mx-auto w-full max-w-md">
	<form>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Car</Field.Legend>
				<Field.Description>Car or Poolcar can be lend for activities.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Car Name</Field.Label>
						<Input id="name" name="name" type="text" placeholder="Enter car type name" required />
					</Field.Field>

					<div class="flex gap-4">
						<Field.Field>
							<Field.Label for="police_number">Police Number</Field.Label>
							<Input
								id="police_number"
								name="police_number"
								type="text"
								placeholder="Enter police number"
								required
							/>
						</Field.Field>

						<Field.Field>
							<Field.Label for="car_type_id">Car Type</Field.Label>
							<Select.Root type="single">
								<Select.Trigger id="car_type_id">Car Type</Select.Trigger>
								<Select.Content>
									<Select.Item value="1">Car Type 1</Select.Item>
									<Select.Item value="2">Car Type 2</Select.Item>
									<Select.Item value="3">Car Type 3</Select.Item>
								</Select.Content>
							</Select.Root>
						</Field.Field>
					</div>

					<Field.Field>
						<Field.Label for="tracker_id">Tracker</Field.Label>
						<Select.Root type="single">
							<Select.Trigger id="tracker_id">Tracker Name</Select.Trigger>
							<Select.Content>
								<Select.Item value="1">Tracker 1</Select.Item>
								<Select.Item value="2">Tracker 2</Select.Item>
								<Select.Item value="3">Tracker 3</Select.Item>
							</Select.Content>
						</Select.Root>
						<Field.Description
							>Enter the tracker that will be used to track the car.</Field.Description
						>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Separator />
			<Field.Set>
				<Field.Group>
					<Field.Field orientation="horizontal">
						<Checkbox id="active" checked />
						<Field.Content>
							<Field.Label for="active">Active</Field.Label>
							<Field.Description>Set the car status to active.</Field.Description>
						</Field.Content>
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
