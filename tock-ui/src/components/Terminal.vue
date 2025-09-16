<script setup>
import Card from '../components/Card.vue';
</script>

<template>
	<Card class="mb-4 relative  h-full flex flex-col" style="background-color: #84A1C4">
		<div class="absolute top-0 left-0 w-full bg-[#607B9B] text-white text-xl px-4 py-1 rounded-t-2xl flex items-center justify-center">
			<span>Terminal</span>
		</div>
		<div class="mt-10 flex-1 overflow-hidden" @wheel.prevent.stop>
			<!-- terminal output goes here; clipped if too large to prevent internal scroll -->
		    <div class="h-full w-full text-white">
				<!-- ...existing code... -->
                 <p>Welcome to the Terminal</p>
                 <p>Type 'help' to see available commands.</p>
                 <p>tock-ui:~ admin$</p>
			</div>
		</div>
    </Card>
</template>