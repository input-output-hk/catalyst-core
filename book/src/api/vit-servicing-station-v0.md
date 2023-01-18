<!-- markdownlint-disable no-inline-html first-line-h1 -->
<!-- We embed the API Docs directly with RAW HTML. -->

<!-- 1. Close the Div amd Main that mdBook creates for us. -->
</main>
</div>

<!-- 2. Define a Style for the API Container (Which fills the available width) -->
<style>
    body {
        display: flex;
        flex-direction: column;
        height: 100vh;
    }
    .api-container {
        flex: 1 0 0;
        overflow: hidden;
    }
</style>

<!-- 3. Use the newly defined style for our API Docs. -->
<div class="api-container">
<!-- 4. Change the apiDescriptionUrl to point to the documentation .json file. -->
<elements-api
  apiDescriptionUrl="../core-vitss-doc/api/v0.yaml"
  router="memory"
  layout="sidebar"
/>
</div>

<!-- 5. Load the OpenAPI Viewer and Stylesheet LAST. -->
<script src="https://unpkg.com/@stoplight/elements/web-components.min.js"></script>
<link rel="stylesheet" href="https://unpkg.com/@stoplight/elements/styles.min.css">

<!-- 6. Re-open the div and main so the rest of the auto-generated content works. -->
<div id="content" class="content">
<main>
