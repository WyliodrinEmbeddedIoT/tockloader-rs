<script setup>
import { defineEmits } from 'vue';
import Card from '../components/Card.vue';
import AppIcon from '../components/AppIcon.vue';
import Terminal from '../components/Terminal.vue';

const emit = defineEmits(['reset-connection']);

function resetConnection() {
	// safe default: emit an event so parent can handle actual reset logic
	emit('reset-connection');
}
</script>


<template>
	<div class="p-6 h-screen flex flex-col overflow-hidden">
		<div class="flex flex-col items-center mb-6">
			<h1 class="text-[#607B9B] text-3xl font-semibold mb-3 mt-5 text-center">DEVICE MANAGER</h1>
		</div>

		<div class="w-full flex-1 overflow-hidden">
			<div class="flex h-full overflow-hidden">
				<div class="w-full md:w-1/2 h-full">
					<Card class="mb-4 relative overflow-hidden h-full flex flex-col">
						<div class="absolute top-0 left-0 w-full bg-[#607B9B] text-white text-xl px-4 py-1 rounded-t-2xl flex items-center justify-center">
							<span>Installed Apps</span>
						</div>
						<RouterLink to="/add-app">
							<button aria-label="Add" class="absolute top-0 right-0 h-9 px-3 bg-[#84A1C4] text-white rounded-tr-2xl flex items-center justify-center z-10 shadow-md">
								<i class="pi pi-file-plus text-lg"></i>
							</button>
						</RouterLink>
						<div class="pt-12 flex-1 overflow-auto">
							<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
								<AppIcon v-for="i in 20" :key="i" class="mx-auto mt-5" />
							</div>
						</div>
					</Card>
				</div>
				<!-- Right column left empty to reserve the right half -->
				<div class="hidden md:flex md:flex-col md:w-1/2 ml-4 h-full">
						<!-- Top half: paragraphs, scrollable if overflow -->
						<div class="h-[40%] overflow-auto p-4 relative">
							<RouterLink to="/">
								<button @click="resetConnection" class="absolute top-2 right-2 text-lg bg-[#84A1C4] text-white px-3 py-2 rounded-xl shadow-xl hover:bg-[#607B9B]">
									Reset connection
								</button>
							</RouterLink>
							<p class="mb-2">Product: BBC micro:bit CMSIS-DAP</p>
							<p class="mb-2">Serial Number: 5738946523759834256986908476543698436984</p>
						</div>
					<!-- Bottom half: terminal area, scrollable -->
					<div class="h-[60%] overflow-auto mt-4">
						<Terminal />
					</div>
				</div>
			</div>
		</div>
	</div>
</template>
