<template>
  <div class="device-display-index">
    <el-card class="header-card">
      <h2>设备展示</h2>
      <p>管理设备在屏幕上的展示内容</p>
    </el-card>

    <el-row :gutter="20" style="margin-top: 20px">
      <el-col :span="24">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>ZecTrix 设备</span>
            </div>
          </template>

          <div class="device-list">
            <div class="device-item" @click="goToZectrix">
              <div class="device-icon">
                <el-icon :size="40"><Monitor /></el-icon>
              </div>
              <div class="device-info">
                <h3>ZecTrix 展示</h3>
                <p>ESP32-S3 设备屏幕展示</p>
              </div>
              <div class="device-arrow">
                <el-icon><ArrowRight /></el-icon>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script lang="ts" setup>
import { useRouter } from 'vue-router'
import { Monitor, ArrowRight } from '@element-plus/icons-vue'

const router = useRouter()

const goToZectrix = () => {
  router.push('/device-display/edge-zectrix')
}
</script>

<style lang="scss" scoped>
.device-display-index {
  padding: 20px;

  .header-card {
    h2 {
      margin: 0 0 10px 0;
    }

    p {
      margin: 0;
      color: #666;
    }
  }

  .card-header {
    font-weight: 600;
  }

  .device-list {
    .device-item {
      display: flex;
      align-items: center;
      padding: 20px;
      border: 1px solid #ebeef5;
      border-radius: 8px;
      cursor: pointer;
      transition: all 0.3s ease;

      &:hover {
        background: #f5f7fa;
        border-color: #5585a9;
      }

      .device-icon {
        width: 60px;
        height: 60px;
        border-radius: 8px;
        background: #ecf5ff;
        color: #5585a9;
        display: flex;
        justify-content: center;
        align-items: center;
      }

      .device-info {
        flex: 1;
        margin-left: 20px;

        h3 {
          margin: 0 0 5px 0;
          font-size: 16px;
        }

        p {
          margin: 0;
          color: #999;
          font-size: 14px;
        }
      }

      .device-arrow {
        color: #c0c4cc;
        font-size: 20px;
      }
    }
  }
}
</style>
