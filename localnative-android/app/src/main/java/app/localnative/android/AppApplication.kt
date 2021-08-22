package app.localnative.android

import android.app.Application
import com.hjq.permissions.XXPermissions
import com.hjq.toast.ToastUtils
import com.hjq.toast.style.WhiteToastStyle


class AppApplication : Application() {
    override fun onCreate() {
        super.onCreate()

        // 初始化吐司工具类
        ToastUtils.init(this, WhiteToastStyle())
        // 当前项目是否已经适配了分区存储的特性
        XXPermissions.setScopedStorage(true);
    }
}