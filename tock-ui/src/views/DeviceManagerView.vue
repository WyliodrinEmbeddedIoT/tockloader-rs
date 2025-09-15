<script setup>
import Card from '../components/Card.vue';
import AppIcon from '../components/AppIcon.vue';
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
						<RouterLink to="/">
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
				<div class="hidden md:block md:w-[48%] ml-4">
				</div>
			</div>
		</div>
	</div>
</template>
