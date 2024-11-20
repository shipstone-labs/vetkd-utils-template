<script lang="ts">
import { Router } from "svelte-router-spa";
import "../store/notes";
import EditNote from "./EditNote.svelte";
import NewNote from "./NewNote.svelte";
import Notes from "./Notes.svelte";
import SidebarLayout from "./SidebarLayout.svelte";

const routes = [
	{
		name: "/new",
		component: NewNote,
	},
	{
		name: "/",
		nestedRoutes: [
			{ name: "index", component: Notes },
			{ name: "edit/:id", component: EditNote },
		],
	},
];
</script>

<SidebarLayout>
  <Router {routes} />
</SidebarLayout>
