<script setup>
import Card from '../components/Card.vue';
</script>

<template>
	<Card class="relative items-center justify-center text-center mt-10 w-3/4 max-w-4xl mx-auto p-20 min-h-80">
		<!-- corner outlines (L-shaped) -->
		<div class="absolute top-3 left-3 w-8 h-8 border-t-4 border-l-4 border-gray-300 rounded-tl-md pointer-events-none" aria-hidden="true"></div>
		<div class="absolute top-3 right-3 w-8 h-8 border-t-4 border-r-4 border-gray-300 rounded-tr-md pointer-events-none" aria-hidden="true"></div>
		<div class="absolute bottom-3 left-3 w-8 h-8 border-b-4 border-l-4 border-gray-300 rounded-bl-md pointer-events-none" aria-hidden="true"></div>
		<div class="absolute bottom-3 right-3 w-8 h-8 border-b-4 border-r-4 border-gray-300 rounded-br-md pointer-events-none" aria-hidden="true"></div>

		<!-- plus inside rounded square -->
		<div class="mb-4 mt-16">
			<span class="inline-flex items-center justify-center w-14 h-14 rounded-lg bg-white text-black border-2 border-black">
				<span class="pi pi-plus text-2xl" aria-hidden="true"></span>
			</span>
		</div>

		<h3 class="text-xl font-medium mb-1">Drag files here to install them on your board</h3>
		<p class="mb-5">Or click here to browse</p>
	</Card>

    <Card class="w-3/4 max-w-4xl mx-auto mt-6 p-4">
		<div class="flex items-center justify-between">
			<span class="text-lg font-medium">App2.tbf</span>
			<button class="text-gray-600 hover:text-red-600" aria-label="Remove App2.tbf">
				<span class="pi pi-times text-xl" aria-hidden="true"></span>
			</button>
		</div>
	</Card>
    <Card class="w-3/4 max-w-4xl mx-auto mt-6 p-4">
		<div class="flex items-center justify-between">
			<span class="text-lg font-medium">App2.tbf</span>
			<button class="text-gray-600 hover:text-red-600" aria-label="Remove App2.tbf">
				<span class="pi pi-times text-xl" aria-hidden="true"></span>
			</button>
		</div>
	</Card>
    <Card class="w-3/4 max-w-4xl mx-auto mt-6 p-4">
		<div class="flex items-center justify-between">
			<span class="text-lg font-medium">App2.tbf</span>
			<button class="text-gray-600 hover:text-red-600" aria-label="Remove App2.tbf">
				<span class="pi pi-times text-xl" aria-hidden="true"></span>
			</button>
		</div>
	</Card>

    <!-- Install button fixed at bottom center -->
    <RouterLink to="/device-manager">
    <div class="fixed inset-x-0 bottom-6 flex justify-center pointer-events-none">
    	<button class="pointer-events-auto bg-[#84A1C4] hover:bg-[#6B8DA4] text-white py-2 px-12 rounded-full shadow-lg min-w-[180px]" aria-label="Install selected apps">
    		Install
    	</button>
    </div>
    </RouterLink>
</template>

