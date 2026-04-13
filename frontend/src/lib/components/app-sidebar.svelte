<script lang="ts">
	import * as Sidebar from "$lib/components/ui/sidebar";
	import HouseIcon from "@lucide/svelte/icons/house";
	import CarIcon from "@lucide/svelte/icons/car";
	import CarFrontIcon from "@lucide/svelte/icons/car-front";
	import MapPinPenIcon from "@lucide/svelte/icons/map-pin-pen";
	import NotebookIcon from "@lucide/svelte/icons/notebook";
	import NotebookPenIcon from "@lucide/svelte/icons/notebook-pen";
	import BookOpenIcon from "@lucide/svelte/icons/book-open";
	import HistoryIcon from "@lucide/svelte/icons/history";
	import MapIcon from "@lucide/svelte/icons/map";
	import UserIcon from "@lucide/svelte/icons/user";
	import ShieldUserIcon from "@lucide/svelte/icons/shield-user";
	import BrickWallShieldIcon from "@lucide/svelte/icons/brick-wall-shield";
	import KeySquareIcon from "@lucide/svelte/icons/key-square";
	import RouteIcon from "@lucide/svelte/icons/route";
	import { resolve } from "$app/paths";
	import type { Component, ComponentProps } from "svelte";
	import type { Url } from "$lib/url";
	const primary_items: {
		title: string;
		url: Url;
		icon: Component;
	}[] = [
		{
			title: "Home",
			url: "/",
			icon: HouseIcon
		},
		{
			title: "Live Tracking",
			url: "/live",
			icon: MapIcon
		},
		{
			title: "Audit",
			url: "/audit",
			icon: BrickWallShieldIcon
		},
		{
			title: "Create New Trip",
			url: "/trip",
			icon: RouteIcon
		}
	];

	const crud_items: {
		title: string;
		url: Url;
		icon: Component;
	}[] = [
		{
			title: "Cars",
			url: "/cars",
			icon: CarIcon
		},
		{
			title: "Car Types",
			url: "/car-types",
			icon: CarFrontIcon
		},
		{
			title: "Car Status",
			url: "/car-status",
			icon: KeySquareIcon
		},
		{
			title: "Trackers",
			url: "/trackers",
			icon: MapPinPenIcon
		},
		{
			title: "Contacts",
			url: "/contacts",
			icon: NotebookIcon
		},
		{
			title: "Contact Types",
			url: "/contact-types",
			icon: NotebookPenIcon
		},
		{
			title: "Activities",
			url: "/activities",
			icon: HistoryIcon
		},
		{
			title: "Activity Types",
			url: "/activity-types",
			icon: BookOpenIcon
		},
		{
			title: "Users",
			url: "/users",
			icon: UserIcon
		},
		{
			title: "User Roles",
			url: "/user-roles",
			icon: ShieldUserIcon
		}
	];

	let { ...restProps }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root collapsible="offcanvas" {...restProps}>
	<Sidebar.Header />
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each primary_items as sidebar_item (sidebar_item.title)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a href={resolve(`${sidebar_item.url}`)} {...props}>
										<sidebar_item.icon />
										<span>{sidebar_item.title}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Management</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each crud_items as sidebar_item (sidebar_item.title)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a href={resolve(sidebar_item.url)} {...props}>
										<sidebar_item.icon />
										<span>{sidebar_item.title}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
	<Sidebar.Footer />
</Sidebar.Root>
