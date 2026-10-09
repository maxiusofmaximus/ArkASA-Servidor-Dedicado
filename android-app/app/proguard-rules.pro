# R8 / ProGuard rules for the ARK ASA companion app.
#
# The UniFFI-generated Kotlin talks to the native library through JNA using
# reflection and JNI symbol names. R8 cannot see those references, so the
# generated classes and JNA's internals must be kept explicitly — otherwise a
# release build compiles fine and then crashes with UnsatisfiedLinkError.

# UniFFI-generated bindings.
-keep class uniffi.** { *; }

# JNA resolves native methods by name through reflection.
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { *; }
-dontwarn java.awt.**

# Keep annotations the generated code relies on.
-keepattributes *Annotation*, InnerClasses, Signature, RuntimeVisibleAnnotations

# The app entry points reachable only from the manifest.
-keep class com.arkasa.mobile.MainActivity { *; }
