package com.github.kr328.clash.compat.helper

import kotlinx.cinterop.*
import java.*

class JavaException(message: String, cause: Exception? = null) : Exception(message, cause)

private val cIOException: LazyClass = LazyClass(
    className = "java/io/IOException"
)

private val mIOExceptionNew: LazyMethod = LazyMethod(
    cfg = LazyMemberConfig(
        className = "java/io/IOException",
        name = "<init>",
        signature = "(Ljava/lang/String;)V",
        isStatic = false,
    ),
)

fun <R>CPointer<JNIEnvVar>.suppressException(block: () -> R): R {
    return functions { env ->
        val throwable: jthrowable? = ExceptionOccurred!!(env)?.also {
            ExceptionClear!!(env)
        }

        try {
            block()
        } finally {
            if (throwable != null) {
                Throw!!(env, throwable)
            }
        }
    }
}

fun Exception.toJavaException(env: CPointer<JNIEnvVar>): jthrowable {
    return env.functions {
        memScoped {
            env.suppressException {
                val args: CArrayPointer<jvalue> = jValues(1) {
                    it[0].l = (message ?: this@toJavaException.toString()).toJString(env)
                }

                NewObjectA!!(env, cIOException.get(env), mIOExceptionNew.get(env), args)!!
            }
        }
    }
}

private val mObjectToString: LazyMethod = LazyMethod(
    cfg = LazyMemberConfig(
        className = "java/lang/Object",
        name = "toString",
        signature = "()Ljava/lang/String;",
        isStatic = false,
    ),
)

fun jthrowable.toKotlinException(env: CPointer<JNIEnvVar>): JavaException {
    return memScoped {
        env.functions {
            env.suppressException {
                val message = CallObjectMethodA!!(
                    env,
                    this@toKotlinException,
                    mObjectToString.get(env),
                    jValues(0)
                )?.toKString(env) ?: "Unknown"

                JavaException(message)
            }
        }
    }
}

fun CPointer<JNIEnvVar>.consumeJavaException(): JavaException? {
    return functions { env ->
        if (ExceptionCheck!!(env).asBoolean()) {
            val throwable = ExceptionOccurred!!(env)!!

            ExceptionClear!!(env)

            throwable.toKotlinException(env)
        } else {
            null
        }
    }
}

fun <R> CPointer<JNIEnvVar>.rethrowKotlinException(block: () -> R): R {
    return functions { env ->
        try {
            val r = block()

            consumeJavaException()?.also {
                throw it
            }

            r
        } catch (e: Exception) {
            if (ExceptionCheck!!(env).asBoolean()) {
                ExceptionClear!!(env)
            }

            throw e
        }
    }
}

fun <R> CPointer<JNIEnvVar>.rethrowJavaException(block: () -> R): R? {
    return try {
        block()
    } catch (e: Exception) {
        memScoped {
            functions { env ->
                Throw!!(env, e.toJavaException(env))
            }
        }

        null
    }
}
