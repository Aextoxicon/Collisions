package com.example.collisions.Repositories

import com.example.collisions.Models.IArtifact
import com.example.collisions.Utils.Result

interface IArtifactRepo {
    suspend fun listAsync(path: String): Result<List<IArtifact>>
    suspend fun searchAsync(query: String): Result<List<IArtifact>>
    suspend fun getAsync(id: String): Result<IArtifact>
    suspend fun deleteAsync(id: String): Result<Boolean>
    suspend fun getContUriAsync(id: String): Result<String>
    suspend fun tryReadTextAsync(id: String): Result<String>
}