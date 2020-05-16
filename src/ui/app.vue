<template>
  <div id="app">
  <h1>Welcome to Timetrack.RS</h1>
  {{ message }} {{username }}
  <h2>Your recent bookings are:</h2>
  <li v-for="item in items">
    {{ item.year }} / {{ item.month }} / {{ item.day }}: {{ item.from.hour }}: {{ item.from.minute }}
  </li>
  <h2>Clock something:</h2>
  </div>
</template>