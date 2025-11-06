<script>
  import "../app.css";
  import UpdateChecker from "$lib/components/UpdateChecker.svelte";
</script>

<!-- Auto-update checker -->
<UpdateChecker />

<!-- Skip to main content link for keyboard navigation -->
<a href="#main-content" class="skip-to-content">
  Skip to main content
</a>

<slot />
