<template>
  <div class="marketplace">
    <div class="search-header">
      <h1>脚本市场</h1>
      <div class="search-box">
        <el-input
          v-model="searchQuery"
          placeholder="搜索脚本或作者名称"
          :prefix-icon="Search"
          clearable
          @keyup.enter="searchScripts"
        />
        <el-button type="primary" @click="searchScripts">搜索</el-button>
      </div>
      <div class="filter-options">
        <el-select v-model="category" placeholder="全部分类" clearable size="small">
          <el-option v-for="item in categories" :key="item" :label="item" :value="item" />
        </el-select>
        <el-select v-model="sortBy" placeholder="排序方式" size="small">
          <el-option label="最新发布" value="latest" />
          <el-option label="最多下载" value="downloads" />
          <el-option label="最高评分" value="rating" />
        </el-select>
      </div>
    </div>
    
    <div class="marketplace-content">
      <div class="marketplace-left">
        <div v-if="loading" class="loading-container">
          <el-skeleton :rows="10" animated />
        </div>
        
        <div v-else-if="scripts.length === 0" class="empty-result">
          <el-empty description="未找到匹配的脚本" />
        </div>
        
        <div v-else class="script-list">
          <div
            v-for="(script, index) in scripts"
            :key="script.id"
            class="script-card"
            :class="{ active: selectedScriptIndex === index }"
            @click="selectScript(index)"
          >
            <div class="script-card-content">
              <h3 class="script-name">{{ script.name }}</h3>
              <p class="script-desc">{{ script.description }}</p>
              <div class="script-meta">
                <span class="author">
                  <el-icon><user /></el-icon>
                  {{ script.author }}
                </span>
                <span class="downloads">
                  <el-icon><download /></el-icon>
                  {{ script.downloads }}
                </span>
                <el-rate
                  v-model="script.rating"
                  disabled
                  text-color="#ff9900"
                  :score-template="script.rating + ''"
                  :show-score="false"
                />
              </div>
            </div>
            <div class="script-card-action">
              <el-button 
                type="primary"
                :loading="script.downloading"
                @click.stop="downloadScript(index)"
              >
                {{ script.downloaded ? '已下载' : '下载' }}
              </el-button>
            </div>
          </div>
          
          <div class="pagination-container">
            <el-pagination
              background
              layout="prev, pager, next"
              :total="100"
              :page-size="10"
              :current-page="currentPage"
              @current-change="handlePageChange"
            />
          </div>
        </div>
      </div>
      
      <div class="marketplace-right" v-if="selectedScript">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <h2>{{ selectedScript.name }}</h2>
              <div class="script-badges">
                <el-tag v-if="selectedScript.verified" type="success">官方验证</el-tag>
                <el-tag v-if="selectedScript.premium" type="warning">赞助专享</el-tag>
              </div>
            </div>
          </template>
          
          <div class="script-details">
            <div class="script-screenshots">
              <el-image
                :src="selectedScript.screenshot || 'https://via.placeholder.com/300x200?text=无预览图'"
                fit="cover"
                class="script-screenshot"
              />
            </div>
            
            <el-descriptions :column="1" border>
              <el-descriptions-item label="作者">{{ selectedScript.author }}</el-descriptions-item>
              <el-descriptions-item label="版本">{{ selectedScript.version }}</el-descriptions-item>
              <el-descriptions-item label="发布日期">{{ formatDate(selectedScript.publishDate) }}</el-descriptions-item>
              <el-descriptions-item label="下载次数">{{ selectedScript.downloads }}</el-descriptions-item>
              <el-descriptions-item label="评分">
                <el-rate v-model="selectedScript.rating" disabled />
                <span>({{ selectedScript.ratingCount || 0 }}人评价)</span>
              </el-descriptions-item>
            </el-descriptions>
            
            <div class="script-section">
              <h3>脚本简介</h3>
              <p>{{ selectedScript.description }}</p>
            </div>
            
            <div class="script-section" v-if="selectedScript.models && selectedScript.models.length">
              <h3>所需模型</h3>
              <div class="model-tags">
                <el-tag
                  v-for="model in selectedScript.models"
                  :key="model"
                  class="model-tag"
                >
                  {{ model }}
                </el-tag>
              </div>
            </div>
            
            <div class="script-section" v-if="selectedScript.changelog">
              <h3>更新日志</h3>
              <div class="changelog">
                <div v-for="(log, idx) in selectedScript.changelog" :key="idx" class="changelog-item">
                  <div class="version">v{{ log.version }}</div>
                  <div class="changes">
                    <ul>
                      <li v-for="(change, cidx) in log.changes" :key="cidx">{{ change }}</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
            
            <div class="script-section" v-if="selectedScript.donation">
              <h3>赞助信息</h3>
              <p>{{ selectedScript.donation }}</p>
            </div>
            
            <div class="button-group">
              <el-button type="primary" @click="downloadScript(selectedScriptIndex)" :loading="selectedScript.downloading">
                {{ selectedScript.downloaded ? '已下载' : '下载脚本' }}
              </el-button>
              <el-button type="success" v-if="selectedScript.downloaded" @click="runScript">
                <el-icon><video-play /></el-icon>
                运行脚本
              </el-button>
            </div>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { 
  Search, 
  User, 
  Download,
  VideoPlay
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';

// 搜索参数
const searchQuery = ref('');
const category = ref('');
const sortBy = ref('latest');
const categories = ['游戏', '办公', '学习', '生活', '社交', '工具'];
const currentPage = ref(1);
const loading = ref(false);
const selectedScriptIndex = ref(0);

// 脚本列表数据
const scripts = ref([
  {
    id: 1,
    name: '通用游戏自动化脚本',
    description: '适用于多种游戏的通用自动化脚本，支持自动完成日常任务、签到、收集资源等功能。',
    author: '脚本大师',
    version: '2.3.1',
    downloads: 12580,
    rating: 4.8,
    ratingCount: 356,
    publishDate: new Date('2023-12-15'),
    verified: true,
    premium: false,
    downloaded: false,
    downloading: false,
    screenshot: 'https://via.placeholder.com/400x300?text=游戏自动化',
    models: ['通用OCR', 'YOLO目标检测'],
    changelog: [
      {
        version: '2.3.1',
        changes: [
          '修复了某些设备上截图失败的问题',
          '优化了识别算法'
        ]
      },
      {
        version: '2.3.0',
        changes: [
          '新增自动收集功能',
          '优化了操作流程',
          '提升了成功率'
        ]
      }
    ],
    donation: '如果你觉得这个脚本对你有帮助，欢迎赞助开发者，以支持后续更新维护。'
  },
  {
    id: 2,
    name: '自动签到助手',
    description: '支持多平台自动签到，包括各大电商平台、社交媒体、学习平台等，解放你的双手！',
    author: '效率专家',
    version: '1.5.2',
    downloads: 8642,
    rating: 4.5,
    ratingCount: 213,
    publishDate: new Date('2023-11-20'),
    verified: true,
    premium: true,
    downloaded: true,
    downloading: false,
    screenshot: 'https://via.placeholder.com/400x300?text=自动签到',
    models: ['通用OCR'],
    changelog: [
      {
        version: '1.5.2',
        changes: [
          '新增5个平台支持',
          '修复了界面识别问题'
        ]
      }
    ],
    donation: '赞助用户可获取更多平台支持和VIP功能。'
  },
  {
    id: 3,
    name: '社交互动自动化',
    description: '自动完成社交平台互动，包括点赞、评论、转发等，提高社交活跃度。',
    author: '社交达人',
    version: '0.9.8',
    downloads: 3254,
    rating: 3.9,
    ratingCount: 87,
    publishDate: new Date('2023-12-01'),
    verified: false,
    premium: false,
    downloaded: false,
    downloading: false,
    models: ['通用OCR', '社交场景识别'],
    donation: '如果你觉得这个脚本对你有帮助，欢迎赞助开发者！'
  }
]);

const selectedScript = computed(() => {
  if (selectedScriptIndex.value === null || selectedScriptIndex.value >= scripts.value.length) return null;
  return scripts.value[selectedScriptIndex.value];
});

// 选择脚本
function selectScript(index) {
  selectedScriptIndex.value = index;
}

// 搜索脚本
function searchScripts() {
  loading.value = true;
  
  // 模拟搜索延迟
  setTimeout(() => {
    // 实际项目中应该调用API进行搜索
    // 这里只是简单模拟搜索结果
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      scripts.value = scripts.value.filter(script => 
        script.name.toLowerCase().includes(query) || 
        script.description.toLowerCase().includes(query) ||
        script.author.toLowerCase().includes(query)
      );
    }
    
    loading.value = false;
    
    // 如果没有结果
    if (scripts.value.length === 0) {
      ElMessage.info('未找到匹配的脚本');
    } else {
      // 默认选中第一个
      selectedScriptIndex.value = 0;
    }
  }, 500);
}

// 下载脚本
function downloadScript(index) {
  const script = scripts.value[index];
  
  if (script.downloaded) {
    ElMessage.info('该脚本已下载');
    return;
  }
  
  if (script.premium) {
    // 检查赞助状态
    ElMessage({
      message: '这是赞助专享脚本，请先赞助作者',
      type: 'warning'
    });
    return;
  }
  
  script.downloading = true;
  
  // 模拟下载过程
  setTimeout(() => {
    script.downloaded = true;
    script.downloading = false;
    ElMessage.success(`${script.name} 下载成功！`);
  }, 2000);
}

// 运行脚本
function runScript() {
  ElMessage.success('脚本已添加到本地列表，请到本地列表页面运行');
}

// 分页
function handlePageChange(page) {
  currentPage.value = page;
  // 实际项目中应该重新请求数据
  // 这里只是简单模拟
  loading.value = true;
  setTimeout(() => {
    loading.value = false;
  }, 500);
}

// 格式化日期
function formatDate(date) {
  if (!date) return '';
  return date.toLocaleDateString('zh-CN');
}
</script>

<style lang="scss" scoped>
.marketplace {
  padding: 16px 0;
  
  .search-header {
    margin-bottom: 20px;
    
    h1 {
      margin-top: 0;
      margin-bottom: 16px;
      font-size: 24px;
      background: var(--primary-gradient);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    
    .search-box {
      display: flex;
      gap: 10px;
      margin-bottom: 10px;
    }
    
    .filter-options {
      display: flex;
      gap: 10px;
    }
  }
  
  .marketplace-content {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    
    @media (max-width: 1200px) {
      grid-template-columns: 1fr;
    }
  }
  
  .marketplace-left {
    .loading-container {
      padding: 16px;
      background: var(--bg-color-soft);
      border-radius: 8px;
    }
    
    .empty-result {
      padding: 40px 0;
    }
    
    .script-list {
      .script-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px;
        margin-bottom: 12px;
        background-color: var(--bg-color-soft);
        border-radius: 8px;
        box-shadow: var(--box-shadow-light);
        cursor: pointer;
        transition: transform 0.2s, box-shadow 0.2s;
        
        &:hover {
          transform: translateY(-2px);
          box-shadow: var(--box-shadow);
        }
        
        &.active {
          background: var(--el-color-primary-light-9);
          border-left: 4px solid var(--el-color-primary);
        }
        
        .script-card-content {
          flex: 1;
          
          .script-name {
            margin-top: 0;
            margin-bottom: 8px;
            font-size: 16px;
          }
          
          .script-desc {
            margin-bottom: 12px;
            color: var(--text-color-secondary);
            overflow: hidden;
            text-overflow: ellipsis;
            display: -webkit-box;
            -webkit-line-clamp: 2;
            -webkit-box-orient: vertical;
          }
          
          .script-meta {
            display: flex;
            align-items: center;
            gap: 16px;
            font-size: 12px;
            color: var(--text-color-secondary);
            
            .author, .downloads {
              display: flex;
              align-items: center;
              gap: 4px;
            }
          }
        }
      }
      
      .pagination-container {
        margin-top: 20px;
        display: flex;
        justify-content: center;
      }
    }
  }
  
  .marketplace-right {
    .card-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      
      h2 {
        margin: 0;
        font-size: 18px;
      }
      
      .script-badges {
        display: flex;
        gap: 8px;
      }
    }
    
    .script-details {
      .script-screenshots {
        margin-bottom: 16px;
        
        .script-screenshot {
          width: 100%;
          height: 200px;
          object-fit: cover;
          border-radius: 4px;
        }
      }
      
      .script-section {
        margin-top: 20px;
        
        h3 {
          font-size: 16px;
          margin-bottom: 10px;
          color: var(--text-color-primary);
        }
        
        .model-tags {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }
        
        .changelog {
          .changelog-item {
            margin-bottom: 10px;
            
            .version {
              font-weight: bold;
              margin-bottom: 4px;
            }
            
            .changes ul {
              margin-top: 0;
              padding-left: 20px;
            }
          }
        }
      }
      
      .button-group {
        margin-top: 20px;
        display: flex;
        gap: 12px;
      }
    }
  }
}
</style> 