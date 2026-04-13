<template>
  <div class="edge-zectrix-display">
    <el-container>
      <!-- 左侧配置区域 -->
      <el-aside width="300px" class="config-aside">
        <el-card class="config-card">
          <template #header>
            <div class="card-header">
              <span>ZecTrix 展示配置</span>
            </div>
          </template>

          <el-form :model="config" label-width="100px" label-position="top">
            <el-form-item label="天气城市">
              <el-input
                v-model="config.weatherCity"
                placeholder="例如: Beijing, Shanghai"
              />
            </el-form-item>

            <el-form-item label="图片刷新间隔(秒)">
              <el-input-number
                v-model="config.imageRefreshInterval"
                :min="5"
                :max="3600"
              />
            </el-form-item>

            <el-form-item>
              <el-button type="primary" @click="saveConfig" style="width: 100%">
                保存配置
              </el-button>
            </el-form-item>
          </el-form>
        </el-card>

        <el-card class="config-card" style="margin-top: 16px">
          <template #header>
            <div class="card-header">
              <span>操作说明</span>
            </div>
          </template>
          <ul class="help-list">
            <li>使用 <kbd>↑</kbd> <kbd>↓</kbd> 方向键翻页</li>
            <li>首页显示当前时间和日期</li>
            <li>第二页显示当地天气</li>
            <li>第三页显示随机图片</li>
          </ul>
        </el-card>
      </el-aside>

      <!-- 右侧展示区域 -->
      <el-main class="display-main">
        <div class="display-container" tabindex="0" @keydown="handleKeyDown">
          <!-- 固定 selector 的展示区域 -->
          <div id="edge-zectrix-display-content" class="display-content">
            <!-- 首页：时间日期 -->
            <div v-show="currentPage === 0" class="page page-datetime">
              <div class="datetime-content">
                <div class="time">{{ currentTime }}</div>
                <div class="date">{{ currentDate }}</div>
                <div class="weekday">{{ currentWeekday }}</div>
              </div>
            </div>

            <!-- 第二页：天气 -->
            <div v-show="currentPage === 1" class="page page-weather">
              <div v-if="weatherData" class="weather-content">
                <div class="weather-icon">
                  <span class="weather-emoji">{{ weatherIcon }}</span>
                </div>
                <div class="weather-temp">{{ weatherData.temperature }}°C</div>
                <div class="weather-desc">{{ weatherDescription }}</div>
                <div class="weather-city">{{ weatherCityName }}</div>
                <div class="weather-detail">
                  <span>风速: {{ weatherData.windspeed }} m/s</span>
                </div>
              </div>
              <div v-else-if="weatherLoading" class="weather-loading">
                <el-icon class="is-loading"><Loading /></el-icon>
                <span>加载天气中...</span>
              </div>
              <div v-else class="weather-error">
                <p>无法加载天气数据</p>
                <el-button size="small" @click="fetchWeather">重试</el-button>
              </div>
            </div>

            <!-- 第三页：ASCII 艺术 -->
            <div v-show="currentPage === 2" class="page page-ascii">
              <pre class="ascii-art">{{ currentAsciiArt }}</pre>
            </div>
          </div>

          <!-- 页码指示器 -->
          <div class="page-indicator">
            <span
              v-for="i in 3"
              :key="i"
              :class="['indicator-dot', { active: currentPage === i - 1 }]"
              @click="goToPage(i - 1)"
            />
          </div>
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script lang="ts" setup>
import { onMounted, onUnmounted, ref, watch, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Loading } from '@element-plus/icons-vue'

// 配置接口
interface EdgeZectrixDisplayConfig {
  weatherCity: string
  imageRefreshInterval: number
}

// Open-Meteo 天气数据接口
interface OpenMeteoWeather {
  temperature: number
  windspeed: number
  weathercode: number
}

interface OpenMeteoGeocoding {
  results?: Array<{
    latitude: number
    longitude: number
    name: string
    country: string
  }>
}

// localStorage key
const STORAGE_KEY = 'edge_zectrix_display_config'

// 状态
const config = ref<EdgeZectrixDisplayConfig>({
  weatherCity: '',
  imageRefreshInterval: 30,
})

const currentPage = ref(0)
const currentTime = ref('')
const currentDate = ref('')
const currentWeekday = ref('')

const weatherData = ref<OpenMeteoWeather | null>(null)
const weatherCityName = ref('')
const weatherLoading = ref(false)
const weatherError = ref(false)

const currentImage = ref('')
const imageLoading = ref(false)

// ASCII 艺术图案库（适配单色显示）
const asciiArtPatterns = [
  `  .----.
  / o  o \\
 |   <>   |
  \\  ..  /
   '----'`,
  `    *    *
   /|  |\\
  / |  | \\
 *--*--*--*
  \\ |  | /
   \\|  |/
    '**'`,
  `  /\\
 /  \\
/    \\
| 机械 |
| 之心 |
 \\____/`,
  `  .------.
  |  --   |
  |  \\/   |
  |       |
  |_______|
  '------'`,
]

const currentAsciiArt = ref('')

// ASCII 艺术定时器
let asciiInterval: ReturnType<typeof setInterval> | null = null

// 更新 ASCII 艺术
const updateAsciiArt = () => {
  const randomIndex = Math.floor(Math.random() * asciiArtPatterns.length)
  currentAsciiArt.value = asciiArtPatterns[randomIndex] ?? ''
}

// 设置 ASCII 艺术定时器
const setupAsciiArtInterval = () => {
  if (asciiInterval) {
    clearInterval(asciiInterval)
  }
  updateAsciiArt()
  asciiInterval = setInterval(updateAsciiArt, config.value.imageRefreshInterval * 1000)
}

// 时间更新定时器
let timeInterval: ReturnType<typeof setInterval> | null = null
// 图片刷新定时器
let imageInterval: ReturnType<typeof setInterval> | null = null

// WMO 天气代码到 ASCII 符号的映射（适配单色显示）
const weatherCodeToIcon: Record<number, string> = {
  0: '*',   // 晴朗
  1: '*',   // 大部晴朗
  2: '~',   // 部分多云
  3: '~',   // 阴天
  45: '=',  // 雾
  48: '=',  // 雾凇
  51: '#',  // 小毛毛雨
  53: '#',  // 中毛毛雨
  55: '#',  // 大毛毛雨
  56: '#',  // 冻毛毛雨
  57: '#',  // 冻毛毛雨
  61: '#',  // 小雨
  63: '#',  // 中雨
  65: '#',  // 大雨
  66: '#',  // 冻雨
  67: '#',  // 冻雨
  71: '*',  // 小雪
  73: '*',  // 中雪
  75: '*',  // 大雪
  77: '*',  // 雪粒
  80: '#',  // 小阵雨
  81: '#',  // 中阵雨
  82: '#',  // 大阵雨
  85: '*',  // 小阵雪
  86: '*',  // 大阵雪
  95: '#',  // 雷暴
  96: '#',  // 雷暴伴小冰雹
  99: '#',  // 雷暴伴大冰雹
}

// WMO 天气代码到中文描述的映射
const weatherCodeToDescription: Record<number, string> = {
  0: '晴朗',
  1: '大部晴朗',
  2: '部分多云',
  3: '阴天',
  45: '有雾',
  48: '雾凇',
  51: '小毛毛雨',
  53: '中毛毛雨',
  55: '大毛毛雨',
  56: '冻毛毛雨',
  57: '大冻毛毛雨',
  61: '小雨',
  63: '中雨',
  65: '大雨',
  66: '冻雨',
  67: '大冻雨',
  71: '小雪',
  73: '中雪',
  75: '大雪',
  77: '雪粒',
  80: '小阵雨',
  81: '中阵雨',
  82: '大阵雨',
  85: '小阵雪',
  86: '大阵雪',
  95: '雷暴',
  96: '雷暴伴小冰雹',
  99: '雷暴伴大冰雹',
}

// 天气 Emoji 图标
const weatherIcon = computed(() => {
  if (!weatherData.value) return ''
  const icon = weatherCodeToIcon[weatherData.value.weathercode]
  return icon || weatherCodeToIcon[0]
})

// 天气描述
const weatherDescription = computed(() => {
  if (!weatherData.value) return ''
  const desc = weatherCodeToDescription[weatherData.value.weathercode]
  return desc || '未知'
})

// 格式化时间（取消秒显示，适配墨水屏刷新）
const updateTime = () => {
  const now = new Date()
  currentTime.value = now.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
  currentDate.value = now.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
  const weekdays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  currentWeekday.value = weekdays[now.getDay()] ?? ''
}

// 加载配置
const loadConfig = () => {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored) {
    try {
      config.value = { ...config.value, ...JSON.parse(stored) }
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }
}

// 保存配置
const saveConfig = () => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config.value))
  ElMessage.success('配置已保存')
  // 刷新天气数据
  if (config.value.weatherCity) {
    fetchWeather()
  }
}

// 获取天气数据
const fetchWeather = async () => {
  if (!config.value.weatherCity) {
    ElMessage.warning('请先配置天气城市')
    return
  }

  weatherLoading.value = true
  weatherError.value = false

  try {
    // 首先通过地理编码 API 获取城市坐标
    const geocodingResponse = await fetch(
      `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(config.value.weatherCity)}&count=1`
    )

    if (!geocodingResponse.ok) {
      throw new Error('Geocoding API error')
    }

    const geocodingData: OpenMeteoGeocoding = await geocodingResponse.json()

    const results = geocodingData.results
    if (!results || results.length === 0) {
      throw new Error('City not found')
    }

    const location = results[0]!
    const { latitude, longitude, name, country } = location
    weatherCityName.value = `${name}, ${country}`

    // 获取天气数据
    const weatherResponse = await fetch(
      `https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current=temperature_2m,weather_code,wind_speed_10m`
    )

    if (!weatherResponse.ok) {
      throw new Error('Weather API error')
    }

    const weatherResult = await weatherResponse.json()
    weatherData.value = {
      temperature: weatherResult.current.temperature_2m,
      windspeed: weatherResult.current.wind_speed_10m,
      weathercode: weatherResult.current.weather_code,
    }
  } catch (e) {
    console.error('Failed to fetch weather:', e)
    weatherError.value = true
    ElMessage.error('获取天气数据失败')
  } finally {
    weatherLoading.value = false
  }
}

// 获取随机图片
const fetchRandomImage = () => {
  // 使用 picsum.photos，添加时间戳避免缓存
  const timestamp = Date.now()
  currentImage.value = `https://picsum.photos/400/300?timestamp=${timestamp}`
}

// 键盘事件处理
const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowUp') {
    prevPage()
  } else if (e.key === 'ArrowDown') {
    nextPage()
  }
}

// 翻页
const prevPage = () => {
  currentPage.value = (currentPage.value - 1 + 3) % 3
}

const nextPage = () => {
  currentPage.value = (currentPage.value + 1) % 3
}

const goToPage = (page: number) => {
  currentPage.value = page
}

// 图片加载完成
const onImageLoad = () => {
  imageLoading.value = false
}

// 图片加载失败
const onImageError = () => {
  // 尝试重新获取图片
  fetchRandomImage()
}

// 设置定时器
const setupIntervals = () => {
  // 更新时间每秒
  timeInterval = setInterval(updateTime, 1000)

  // 更新图片定时器
  if (imageInterval) {
    clearInterval(imageInterval)
  }
  imageInterval = setInterval(fetchRandomImage, config.value.imageRefreshInterval * 1000)
}

// 监听配置变化
watch(
  () => config.value.imageRefreshInterval,
  () => {
    if (imageInterval) {
      clearInterval(imageInterval)
    }
    imageInterval = setInterval(fetchRandomImage, config.value.imageRefreshInterval * 1000)
  }
)

// 组件挂载
onMounted(() => {
  loadConfig()
  updateTime()
  fetchRandomImage()

  if (config.value.weatherCity) {
    fetchWeather()
  }

  setupIntervals()
  setupAsciiArtInterval()

  // 聚焦到展示容器以接收键盘事件
  const container = document.querySelector('.display-container') as HTMLElement
  container?.focus()
})

// 组件卸载
onUnmounted(() => {
  if (timeInterval) {
    clearInterval(timeInterval)
  }
  if (imageInterval) {
    clearInterval(imageInterval)
  }
  if (asciiInterval) {
    clearInterval(asciiInterval)
  }
})
</script>

<style lang="scss" scoped>
.edge-zectrix-display {
  height: 100%;
  padding: 20px;
}

.config-aside {
  padding-right: 20px;
}

.config-card {
  .card-header {
    font-weight: 600;
  }

  .help-list {
    margin: 0;
    padding-left: 20px;
    color: #666;
    font-size: 14px;

    li {
      margin-bottom: 8px;
    }

    kbd {
      background: #eee;
      border: 1px solid #ccc;
      border-radius: 3px;
      padding: 2px 6px;
      font-family: monospace;
    }
  }
}

.display-main {
  padding: 0;
  display: flex;
  justify-content: center;
  align-items: center;
}

.display-container {
  width: 100%;
  height: 100%;
  max-width: 450px;
  max-height: 350px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;

  &:focus {
    outline: none;
  }
}

.display-content {
  flex: 1;
  position: relative;
}

.page {
  position: absolute;
  inset: 0;
  display: flex;
  justify-content: center;
  align-items: center;
}

// 首页：时间日期（单色适配）
.page-datetime {
  background: #fff;
  color: #000;

  .datetime-content {
    text-align: center;
    font-family: 'Courier New', Consolas, monospace;

    .time {
      font-size: 64px;
      font-weight: 700;
      letter-spacing: 4px;
    }

    .date {
      font-size: 20px;
      margin-top: 16px;
      font-weight: 500;
    }

    .weekday {
      font-size: 16px;
      margin-top: 8px;
    }
  }
}

// 第二页：天气（单色适配）
.page-weather {
  background: #fff;
  color: #000;

  .weather-content {
    text-align: center;
    color: #000;
    font-family: 'Courier New', Consolas, monospace;

    .weather-icon .weather-emoji {
      font-size: 56px;
      line-height: 1;
    }

    .weather-temp {
      font-size: 40px;
      font-weight: 400;
    }

    .weather-desc {
      font-size: 16px;
      text-transform: capitalize;
      margin-top: 5px;
    }

    .weather-city {
      font-size: 18px;
      margin-top: 10px;
      font-weight: 500;
    }

    .weather-detail {
      margin-top: 15px;
      font-size: 14px;
      color: #000;
      display: flex;
      gap: 20px;
      justify-content: center;
    }
  }

  .weather-loading,
  .weather-error {
    text-align: center;
    color: #000;

    .el-icon {
      font-size: 24px;
      margin-bottom: 10px;
    }
  }
}

// 第三页：ASCII 艺术（单色适配）
.page-ascii {
  background: #fff;
  color: #000;
  padding: 10px;

  .ascii-art {
    font-family: 'Courier New', Consolas, monospace;
    font-size: 10px;
    line-height: 1.1;
    white-space: pre;
    text-align: center;
    margin: 0;
    color: #000;
  }
}

// 页码指示器（单色适配）
.page-indicator {
  position: absolute;
  bottom: 10px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 8px;

  .indicator-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.3);
    cursor: pointer;
    transition: all 0.3s ease;

    &.active {
      background: #000;
      transform: scale(1.2);
    }

    &:hover {
      background: rgba(0, 0, 0, 0.6);
    }
  }
}
</style>
