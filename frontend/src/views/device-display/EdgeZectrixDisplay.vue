<template>
  <div class="edge-zectrix-display">
    <el-container>
      <!-- 左侧配置区域 -->
      <el-aside class="config-aside" width="300px">
        <el-card class="config-card">
          <template #header>
            <div class="card-header">
              <span>ZecTrix 展示配置</span>
            </div>
          </template>

          <el-form :model="config" label-position="top" label-width="100px">
            <el-form-item label="天气城市">
              <el-input
                  v-model="config.weatherCity"
                  placeholder="例如: Beijing, Shanghai"
              />
            </el-form-item>

            <el-form-item>
              <el-button style="width: 100%" type="primary" @click="saveConfig">
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
            <li>每5分钟自动切换下一页</li>
            <li>首页显示当前时间和日期</li>
            <li>第二页显示当地天气</li>
            <li>第三页显示未来3天天气预报</li>
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
                  <el-icon :size="56">
                    <component :is="weatherIconComponent"/>
                  </el-icon>
                </div>
                <div class="weather-temp">{{ weatherData.temperature }}°C</div>
                <div class="weather-desc">{{ weatherDescription }}</div>
                <div class="weather-city">{{ weatherCityName }}</div>
                <div class="weather-detail">
                  <span>风速: {{ weatherData.windspeed }} m/s</span>
                </div>
              </div>
              <div v-else-if="weatherLoading" class="weather-loading">
                <el-icon class="is-loading">
                  <Loading/>
                </el-icon>
                <span>加载天气中...</span>
              </div>
              <div v-else class="weather-error">
                <p>无法加载天气数据</p>
                <el-button size="small" @click="fetchWeather">重试</el-button>
              </div>
            </div>

            <!-- 第三页：3天天气预报 -->
            <div v-show="currentPage === 2" class="page page-forecast">
              <div class="forecast-content">
                <h3 class="forecast-title">3天天气预报</h3>
                <div class="forecast-list">
                  <div v-for="day in forecastData.slice(0, 3)" :key="day.dateRaw" class="forecast-item">
                    <div class="forecast-day">{{ day.date }}</div>
                    <el-icon class="forecast-icon">
                      <component :is="iconMap[day.icon]"/>
                    </el-icon>
                    <div class="forecast-temps">
                      <span class="temp-max">{{ day.tempMax }}°</span>
                      <span class="temp-min">{{ day.tempMin }}°</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

          </div>
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script lang="ts" setup>
import {computed, onMounted, onUnmounted, ref} from 'vue'
import {ElMessage} from 'element-plus'
import {Cloudy, ColdDrink, Drizzling, Lightning, Loading, PartlyCloudy, Pouring, Sunny} from '@element-plus/icons-vue'
import {createModuleConfig, getModuleConfig, updateModuleConfig} from '@/api/moduleConfigs'

// 模块名称前缀
const MODULE_NAME = 'frontend_edge_zectrix_display'

// 配置接口
interface EdgeZectrixDisplayConfig {
  weatherCity: string
}

// 配置 Schema
const CONFIG_SCHEMA = {
  type: 'object',
  properties: {
    weatherCity: {
      type: 'string',
      description: '天气城市名称',
    },
  },
  required: ['weatherCity'],
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

// Open-Meteo 每日预报数据接口
interface OpenMeteoDailyForecast {
  time: string[]
  weather_code: number[]
  temperature_2m_max: number[]
  temperature_2m_min: number[]
}

// 每日天气预报项
interface DailyForecast {
  date: string
  dateRaw: string
  icon: string
  description: string
  tempMax: number
  tempMin: number
}

// 状态
const config = ref<EdgeZectrixDisplayConfig>({
  weatherCity: '',
})

// 配置 ID
const configId = ref<string>('')

const currentPage = ref(0)
const currentTime = ref('')
const currentDate = ref('')
const currentWeekday = ref('')

const weatherData = ref<OpenMeteoWeather | null>(null)
const weatherCityName = ref('')
const weatherLoading = ref(false)
const weatherError = ref(false)
const forecastData = ref<DailyForecast[]>([])

// 时间更新定时器
let timeInterval: ReturnType<typeof setInterval> | null = null
// 页面自动切换定时器
let pageInterval: ReturnType<typeof setInterval> | null = null

// WMO 天气代码到 Element Plus 图标名的映射
const weatherCodeToIcon: Record<number, string> = {
  0: 'Sunny',           // 晴朗
  1: 'Sunny',           // 大部晴朗
  2: 'PartlyCloudy',    // 部分多云
  3: 'Cloudy',          // 阴天
  45: 'Cloudy',         // 雾
  48: 'Cloudy',         // 雾凇
  51: 'Drizzling',      // 小毛毛雨
  53: 'Drizzling',      // 中毛毛雨
  55: 'Drizzling',      // 大毛毛雨
  56: 'Drizzling',      // 冻毛毛雨
  57: 'Drizzling',      // 冻毛毛雨
  61: 'Drizzling',      // 小雨
  63: 'Pouring',        // 中雨
  65: 'Pouring',        // 大雨
  66: 'Drizzling',      // 冻雨
  67: 'Pouring',        // 冻雨
  71: 'ColdDrink',      // 小雪
  73: 'ColdDrink',      // 中雪
  75: 'ColdDrink',      // 大雪
  77: 'ColdDrink',      // 雪粒
  80: 'Pouring',        // 小阵雨
  81: 'Pouring',        // 中阵雨
  82: 'Pouring',        // 大阵雨
  85: 'ColdDrink',      // 小阵雪
  86: 'ColdDrink',      // 大阵雪
  95: 'Lightning',      // 雷暴
  96: 'Lightning',      // 雷暴伴小冰雹
  99: 'Lightning',      // 雷暴伴大冰雹
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

// 天气图标组件映射
const iconMap: Record<string, any> = {
  Sunny,
  PartlyCloudy,
  Cloudy,
  Drizzling,
  Pouring,
  Lightning,
  ColdDrink,
}

// 天气图标组件
const weatherIconComponent = computed(() => {
  if (!weatherData.value) return Sunny
  const iconName = weatherCodeToIcon[weatherData.value.weathercode] || 'Sunny'
  return iconMap[iconName] || Sunny
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
const loadConfig = async () => {
  try {
    const moduleConfig = await getModuleConfig(MODULE_NAME)
    config.value = {...config.value, ...moduleConfig.data.config_value as EdgeZectrixDisplayConfig}
    configId.value = moduleConfig.data.id
  } catch {
    // 配置不存在，创建默认配置
    try {
      const newConfig = await createModuleConfig({
        module_name: MODULE_NAME,
        config_value: config.value,
        json_schema: CONFIG_SCHEMA,
      })
      configId.value = newConfig.data.id
    } catch (e) {
      console.error('Failed to create config:', e)
    }
  }
}

// 保存配置
const saveConfig = async () => {
  if (!configId.value) {
    ElMessage.error('配置未加载')
    return
  }
  try {
    await updateModuleConfig(configId.value, {config_value: config.value})
    ElMessage.success('配置已保存')
    // 刷新天气数据
    if (config.value.weatherCity) {
      fetchWeather()
    }
  } catch (e) {
    console.error('Failed to save config:', e)
    ElMessage.error('保存配置失败')
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
    const {latitude, longitude, name, country} = location
    weatherCityName.value = `${name}, ${country}`

    // 获取天气数据
    const weatherResponse = await fetch(
        `https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current=temperature_2m,weather_code,wind_speed_10m&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto`
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

    // 解析每日预报数据
    const daily = weatherResult.daily as OpenMeteoDailyForecast
    const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
    forecastData.value = daily.time.map((dateStr, index) => {
      const date = new Date(dateStr)
      const weatherCode = daily.weather_code[index] ?? 0
      return {
        date: weekdays[date.getDay()] ?? '',
        dateRaw: dateStr,
        icon: weatherCodeToIcon[weatherCode] || 'Sunny',
        description: weatherCodeToDescription[weatherCode] || '未知',
        tempMax: daily.temperature_2m_max[index] ?? 0,
        tempMin: daily.temperature_2m_min[index] ?? 0,
      }
    })
  } catch (e) {
    console.error('Failed to fetch weather:', e)
    weatherError.value = true
    ElMessage.error('获取天气数据失败')
  } finally {
    weatherLoading.value = false
  }
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
const TOTAL_PAGES = 3

const prevPage = () => {
  currentPage.value = (currentPage.value - 1 + TOTAL_PAGES) % TOTAL_PAGES
}

const nextPage = () => {
  currentPage.value = (currentPage.value + 1) % TOTAL_PAGES
}

// 组件挂载
onMounted(async () => {
  await loadConfig()
  updateTime()

  if (config.value.weatherCity) {
    fetchWeather()
  }

  // 更新时间每分钟（墨水屏不需要每秒刷新）
  timeInterval = setInterval(updateTime, 60000)

  // 每5分钟自动切换到下一页
  pageInterval = setInterval(nextPage, 5 * 60 * 1000)

  // 聚焦到展示容器以接收键盘事件
  const container = document.querySelector('.display-container') as HTMLElement
  container?.focus()
})

// 组件卸载
onUnmounted(() => {
  if (timeInterval) {
    clearInterval(timeInterval)
  }
  if (pageInterval) {
    clearInterval(pageInterval)
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

// 第三页：3天预报
.page-forecast {
  background: #fff;
  color: #000;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;

  .forecast-content {
    text-align: center;
    font-family: 'Courier New', Consolas, monospace;
    padding: 20px;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .forecast-title {
    font-size: 28px;
    margin-bottom: 24px;
  }

  .forecast-list {
    display: flex;
    flex-direction: row;
    justify-content: space-around;
    flex: 1;
  }

  .forecast-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    flex: 1;
    padding: 0 16px;
    border-right: 1px solid #ccc;

    &:last-child {
      border-right: none;
    }

    .forecast-day {
      font-weight: 500;
      font-size: 24px;
    }

    .forecast-icon {
      font-size: 48px;
    }

    .forecast-temps {
      display: flex;
      flex-direction: column;
      gap: 4px;
      font-size: 24px;

      .temp-max {
        color: #000;
      }

      .temp-min {
        color: #666;
      }
    }
  }
}
</style>
