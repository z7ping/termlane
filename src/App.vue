<template>
  <div class="min-h-screen bg-gray-900 text-white">
    <header class="bg-gray-800 p-4">
      <h1 class="text-2xl font-bold">XTerminal Pro</h1>
    </header>
    <main class="p-4">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div class="bg-gray-800 p-4 rounded-lg" v-for="i in 6" :key="i">
          <h3 class="text-lg font-semibold mb-2">Server {{ i }}</h3>
          <div class="bg-black p-2 rounded h-32 font-mono text-sm text-green-400">
            <div>$ Welcome to XTerminal Pro</div>
            <div>$ Server is ready...</div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script>
export default {
  name: 'App'
}
</script>

<style>
body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
}
</style>