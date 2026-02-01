<script lang="ts">
	import * as Sidebar from "$lib/components/ui/sidebar";
	import * as Breadcrumb from "$lib/components/ui/breadcrumb";
	import { page } from "$app/state";
	import type { Url } from "$lib/url";

	interface BreadcrumbItem {
		title: string;
		href: Url;
		subTitle?: string;
	}

	const breadcrumbItems: BreadcrumbItem[] = [
		{ title: "Home", href: "/" },
		{ title: "Cars", href: "/cars" },
		{ title: "Car Types", href: "/car-types" },
		{ title: "Trackers", href: "/trackers" },
		{ title: "Contacts", href: "/contacts" },
		{ title: "Contact Types", href: "/contact-types" },
		{ title: "Activity", href: "/activity" },
		{ title: "History", href: "/history" },
		{ title: "Live Tracking", href: "/live" }
	];

	// Generate breadcrumbs from current route (reactive)
	const currentBreadcrumb = $derived.by(() => {
		const path = page.url.pathname;
		let result: BreadcrumbItem = breadcrumbItems[0];

		// Find the matching breadcrumb (iterate in reverse to match longest path first)
		for (let i = breadcrumbItems.length - 1; i >= 0; i--) {
			const breadcrumbItem = breadcrumbItems[i];
			if (
				path === breadcrumbItem.href ||
				(breadcrumbItem.href !== "/" && path.startsWith(breadcrumbItem.href))
			) {
				result = { ...breadcrumbItem };

				// Count slashes in the path
				const slashCount = (path.match(/\//g) || []).length;

				// If there are 3 slashes total, check the segment after the second slash
				if (slashCount === 2) {
					const segments = path.split("/").filter((s) => s.length > 0);
					if (segments.length >= 1) {
						const lastSegment = segments[segments.length - 1];

						// Check if it's "create" or a number
						if (lastSegment.toLowerCase() === "create") {
							result.subTitle = `Create ${breadcrumbItem.title.slice(0, -1)}`; // Remove plural 's'
						} else if (!isNaN(Number(lastSegment))) {
							result.subTitle = `Modify ${breadcrumbItem.title.slice(0, -1)}`; // Remove plural 's'
						}
					}
				}
				break;
			}
		}

		return result;
	});
</script>

<header class="sticky z-10 flex items-center gap-4 px-4 py-2">
	<Sidebar.Trigger class="-ms-1" />
	<Breadcrumb.Root>
		<Breadcrumb.List>
			<Breadcrumb.Item>
				<Breadcrumb.Link href={currentBreadcrumb.href}>{currentBreadcrumb.title}</Breadcrumb.Link>
			</Breadcrumb.Item>
			{#if currentBreadcrumb.subTitle}
				<Breadcrumb.Separator />
				<Breadcrumb.Item>
					<Breadcrumb.Link href={page.url.pathname}>{currentBreadcrumb.subTitle}</Breadcrumb.Link>
				</Breadcrumb.Item>
			{/if}
		</Breadcrumb.List>
	</Breadcrumb.Root>
</header>
