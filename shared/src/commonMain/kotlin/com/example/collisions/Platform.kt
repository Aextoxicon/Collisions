package com.example.collisions

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform