<template>
  <AppDialog
    :open="open"
    :title="mode === 'edit' ? '编辑脚本' : '新建脚本'"
    description="按信息分组编辑，避免长表单混在一列。"
    :width-class="dialogWidthClass"
    @close="$emit('close')"
  >
    <form v-if="form" class="grid gap-5 lg:grid-cols-[220px_minmax(0,1fr)]" :class="formClass" @submit.prevent="submit">
      <aside class="space-y-2">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="app-list-item"
          :data-testid="`script-dialog-tab-${tab.id}`"
          :class="{ 'app-list-item-active': activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ tab.label }}</p>
          <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ tab.description }}</p>
        </button>

        <SurfacePanel tone="muted" padding="sm" class="hidden lg:block">
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">编辑摘要</p>
          <div class="mt-3 space-y-3 text-sm">
            <div class="flex items-center justify-between gap-3">
              <span class="text-[var(--app-text-faint)]">名称</span>
              <span class="max-w-[120px] truncate text-right text-[var(--app-text-strong)]">{{ form.data.name || '未命名脚本' }}</span>
            </div>
            <div class="flex items-center justify-between gap-3">
              <span class="text-[var(--app-text-faint)]">类型</span>
              <span class="text-[var(--app-text-strong)]">{{ scriptTypeLabel }}</span>
            </div>
            <div class="flex items-center justify-between gap-3">
              <span class="text-[var(--app-text-faint)]">运行时</span>
              <span class="text-[var(--app-text-strong)]">{{ runtimeLabel }}</span>
            </div>
            <div class="flex items-center justify-between gap-3">
              <span class="text-[var(--app-text-faint)]">允许克隆</span>
              <span class="text-[var(--app-text-strong)]">{{ form.data.allowClone ? '是' : '否' }}</span>
            </div>
          </div>
        </SurfacePanel>
      </aside>

      <div class="space-y-5">
        <template v-if="activeTab === 'basic'">
          <div class="grid gap-4 xl:grid-cols-[minmax(0,1.4fr)_320px]">
            <SurfacePanel tone="muted" padding="sm" class="space-y-4">
              <div>
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">基础信息</p>
                <p class="text-xs text-[var(--app-text-faint)]">面向用户展示的主要说明和运行信息。</p>
              </div>

              <div class="grid gap-4 md:grid-cols-2">
                <label class="space-y-2 md:col-span-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">脚本名称</span>
                  <input
                    v-model.trim="form.data.name"
                    class="app-input"
                    data-testid="script-basic-name"
                    maxlength="40"
                    placeholder="例如：每日清体力"
                  />
                </label>

                <label class="space-y-2 md:col-span-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">描述</span>
                  <textarea
                    v-model="descriptionValue"
                    class="app-textarea min-h-[130px]"
                    data-testid="script-basic-description"
                    maxlength="240"
                    placeholder="简述脚本作用、运行前提和风险提示。"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">运行时</span>
                  <AppSelect v-model="form.data.runtimeType" :options="runtimeOptions" test-id="script-basic-runtime-type" />
                </label>

                <label class="space-y-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">包名</span>
                  <input
                    v-model.trim="pkgNameValue"
                    class="app-input"
                    data-testid="script-basic-package-name"
                    maxlength="80"
                    placeholder="com.example.app"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">版本名称</span>
                  <input
                    v-model.trim="form.data.verName"
                    class="app-input"
                    data-testid="script-basic-version-name"
                    maxlength="20"
                    placeholder="0.1.0"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">版本号</span>
                  <input
                    v-model.number="form.data.verNum"
                    class="app-input"
                    data-testid="script-basic-version-num"
                    min="1"
                    type="number"
                  />
                </label>
              </div>
            </SurfacePanel>

            <SurfacePanel tone="muted" padding="sm" class="space-y-4">
              <div>
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">发布摘要</p>
                <p class="text-xs text-[var(--app-text-faint)]">右侧只保留关键状态，避免主表单过长。</p>
              </div>

              <div class="space-y-3 text-sm">
                <div class="rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                  <p class="text-xs text-[var(--app-text-faint)]">作者</p>
                  <p class="mt-1 font-medium text-[var(--app-text-strong)]">{{ form.data.userName || 'Local User' }}</p>
                </div>

                <div class="rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                  <p class="text-xs text-[var(--app-text-faint)]">脚本类型</p>
                  <p class="mt-1 font-medium text-[var(--app-text-strong)]">{{ scriptTypeLabel }}</p>
                </div>

                <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                  <input
                    v-model="form.data.allowClone"
                    type="checkbox"
                    class="h-4 w-4"
                    data-testid="script-basic-allow-clone"
                    style="accent-color: var(--app-accent)"
                  />
                  <span>
                    <span class="block text-sm font-medium text-[var(--app-text-strong)]">允许克隆</span>
                    <span class="block text-xs text-[var(--app-text-faint)]">关闭后，其他用户只能查看脚本信息，不能直接复制到本地。</span>
                  </span>
                </label>
              </div>
            </SurfacePanel>
          </div>
        </template>

        <template v-else-if="activeTab === 'models'">
          <div class="grid gap-4 xl:grid-cols-3">
            <SurfacePanel tone="muted" padding="sm" class="space-y-4">
              <div class="space-y-1">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">图像检测模型</p>
                <p class="text-xs text-[var(--app-text-faint)]">用于图像目标识别。</p>
              </div>
              <AppSelect
                :model-value="imgDetKind"
                :options="detectorOptions"
                test-id="script-models-img-det-kind"
                @update:model-value="setDetectorKind('imgDetModel', $event)"
              />
              <template v-if="imgDetKind === 'Yolo11' && form.data.imgDetModel && 'Yolo11' in form.data.imgDetModel">
                <ModelBaseFields
                  :model="form.data.imgDetModel.Yolo11.baseModel"
                  path-placeholder="例如：D:\\models\\img-det.onnx"
                  test-id-prefix="script-models-img-det-base"
                />
                <div class="grid gap-4 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">类别数量</span>
                    <input
                      v-model.number="form.data.imgDetModel.Yolo11.classCount"
                      class="app-input"
                      data-testid="script-models-img-det-class-count"
                      min="1"
                      type="number"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">标签路径</span>
                    <input
                      v-model.trim="labelPathImg"
                      class="app-input"
                      data-testid="script-models-img-det-label-path"
                      placeholder="例如：D:\\models\\labels.yaml"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">置信度阈值</span>
                    <input
                      v-model.number="form.data.imgDetModel.Yolo11.confidenceThresh"
                      class="app-input"
                      data-testid="script-models-img-det-confidence"
                      max="1"
                      min="0"
                      step="0.01"
                      type="number"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">IOU 阈值</span>
                    <input
                      v-model.number="form.data.imgDetModel.Yolo11.iouThresh"
                      class="app-input"
                      data-testid="script-models-img-det-iou"
                      max="1"
                      min="0"
                      step="0.01"
                      type="number"
                    />
                  </label>
                </div>
              </template>
              <template v-else-if="imgDetKind === 'PaddleDbNet' && form.data.imgDetModel && 'PaddleDbNet' in form.data.imgDetModel">
                <ModelBaseFields
                  :model="form.data.imgDetModel.PaddleDbNet.baseModel"
                  path-placeholder="例如：D:\\models\\dbnet.onnx"
                  test-id-prefix="script-models-img-det-base"
                />
                <div class="grid gap-4 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">二值化阈值</span>
                    <input v-model.number="form.data.imgDetModel.PaddleDbNet.dbThresh" class="app-input" max="1" min="0" step="0.01" type="number" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">框阈值</span>
                    <input v-model.number="form.data.imgDetModel.PaddleDbNet.dbBoxThresh" class="app-input" max="1" min="0" step="0.01" type="number" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">扩张比例</span>
                    <input v-model.number="form.data.imgDetModel.PaddleDbNet.unclipRatio" class="app-input" min="0" step="0.1" type="number" />
                  </label>
                  <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                    <input v-model="form.data.imgDetModel.PaddleDbNet.useDilation" type="checkbox" class="h-4 w-4" style="accent-color: var(--app-accent)" />
                    <span>
                      <span class="block text-sm font-medium text-[var(--app-text-strong)]">启用膨胀</span>
                      <span class="block text-xs text-[var(--app-text-faint)]">适合文本区域边缘需要扩张的场景。</span>
                    </span>
                  </label>
                </div>
              </template>
              <template v-else-if="imgDetKind === 'Yolo26'">
                <ModelBaseFields :model="imgYolo26.baseModel" path-placeholder="例如：D:\\models\\img-det-yolo26.onnx" />
                <div class="grid gap-4 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">类别数量</span>
                    <input v-model.number="imgYolo26.classCount" class="app-input" min="1" type="number" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">标签路径</span>
                    <input v-model.trim="imgYolo26.labelPath" class="app-input" placeholder="例如：D:\\models\\labels.yaml" />
                  </label>
                </div>
                <p class="text-xs text-[rgb(180,83,9)]">Yolo26 目前是前端预留配置，后端未接入，当前不能保存为可运行模型。</p>
              </template>
            </SurfacePanel>

            <SurfacePanel tone="muted" padding="sm" class="space-y-4">
              <div class="space-y-1">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">文本检测模型</p>
                <p class="text-xs text-[var(--app-text-faint)]">用于 OCR 前的文本区域定位。</p>
              </div>
              <AppSelect
                :model-value="txtDetKind"
                :options="detectorOptions"
                test-id="script-models-txt-det-kind"
                @update:model-value="setDetectorKind('txtDetModel', $event)"
              />
              <template v-if="txtDetKind === 'Yolo11' && form.data.txtDetModel && 'Yolo11' in form.data.txtDetModel">
                <ModelBaseFields
                  :model="form.data.txtDetModel.Yolo11.baseModel"
                  path-placeholder="例如：D:\\models\\txt-det.onnx"
                  test-id-prefix="script-models-txt-det-base"
                />
                <div class="grid gap-4 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">类别数量</span>
                    <input v-model.number="form.data.txtDetModel.Yolo11.classCount" class="app-input" min="1" type="number" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">标签路径</span>
                    <input v-model.trim="labelPathTxt" class="app-input" placeholder="例如：D:\\models\\labels.yaml" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">文本类别索引</span>
                    <input v-model.number="txtIdxValue" class="app-input" min="0" type="number" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">置信度阈值</span>
                    <input v-model.number="form.data.txtDetModel.Yolo11.confidenceThresh" class="app-input" max="1" min="0" step="0.01" type="number" />
                  </label>
                  <label class="space-y-2 md:col-span-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">IOU 阈值</span>
                    <input v-model.number="form.data.txtDetModel.Yolo11.iouThresh" class="app-input" max="1" min="0" step="0.01" type="number" />
                  </label>
                </div>
              </template>
              <template v-else-if="txtDetKind === 'PaddleDbNet' && form.data.txtDetModel && 'PaddleDbNet' in form.data.txtDetModel">
                <ModelBaseFields
                  :model="form.data.txtDetModel.PaddleDbNet.baseModel"
                  path-placeholder="例如：D:\\models\\ocr-dbnet.onnx"
                  test-id-prefix="script-models-txt-det-base"
                />
                <div class="grid gap-4 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">二值化阈值</span>
                    <input
                      v-model.number="form.data.txtDetModel.PaddleDbNet.dbThresh"
                      class="app-input"
                      data-testid="script-models-txt-det-db-thresh"
                      max="1"
                      min="0"
                      step="0.01"
                      type="number"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">框阈值</span>
                    <input
                      v-model.number="form.data.txtDetModel.PaddleDbNet.dbBoxThresh"
                      class="app-input"
                      data-testid="script-models-txt-det-db-box-thresh"
                      max="1"
                      min="0"
                      step="0.01"
                      type="number"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">扩张比例</span>
                    <input
                      v-model.number="form.data.txtDetModel.PaddleDbNet.unclipRatio"
                      class="app-input"
                      data-testid="script-models-txt-det-unclip-ratio"
                      min="0"
                      step="0.1"
                      type="number"
                    />
                  </label>
                  <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                    <input
                      v-model="form.data.txtDetModel.PaddleDbNet.useDilation"
                      type="checkbox"
                      class="h-4 w-4"
                      data-testid="script-models-txt-det-use-dilation"
                      style="accent-color: var(--app-accent)"
                    />
                    <span>
                      <span class="block text-sm font-medium text-[var(--app-text-strong)]">启用膨胀</span>
                      <span class="block text-xs text-[var(--app-text-faint)]">对弱文本边缘更友好，但可能带来额外噪点。</span>
                    </span>
                  </label>
                </div>
              </template>
              <template v-else-if="txtDetKind === 'Yolo26'">
                <ModelBaseFields :model="txtYolo26.baseModel" path-placeholder="例如：D:\\models\\txt-det-yolo26.onnx" />
                <div class="grid gap-4 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">类别数量</span>
                    <input v-model.number="txtYolo26.classCount" class="app-input" min="1" type="number" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">文本类别索引</span>
                    <input v-model.number="txtYolo26.txtIdx" class="app-input" min="0" type="number" />
                  </label>
                </div>
                <p class="text-xs text-[rgb(180,83,9)]">Yolo26 目前是前端预留配置，后端未接入，当前不能保存为可运行模型。</p>
              </template>
            </SurfacePanel>

            <SurfacePanel tone="muted" padding="sm" class="space-y-4">
              <div class="space-y-1">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">文本识别模型</p>
                <p class="text-xs text-[var(--app-text-faint)]">用于 OCR 的字符识别阶段。</p>
              </div>
              <AppSelect
                :model-value="txtRecKind"
                :options="recognizerOptions"
                test-id="script-models-txt-rec-kind"
                @update:model-value="setRecognizerKind($event)"
              />
              <template v-if="txtRecKind === 'PaddleCrnn' && form.data.txtRecModel && 'PaddleCrnn' in form.data.txtRecModel">
                <ModelBaseFields
                  :model="form.data.txtRecModel.PaddleCrnn.baseModel"
                  path-placeholder="例如：D:\\models\\ocr-rec.onnx"
                  test-id-prefix="script-models-txt-rec-base"
                />
                <label class="space-y-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">字典路径</span>
                  <input
                    v-model.trim="dictPathValue"
                    class="app-input"
                    data-testid="script-models-txt-rec-dict-path"
                    placeholder="例如：D:\\models\\keys.txt"
                  />
                </label>
              </template>
            </SurfacePanel>
          </div>

          <div v-if="hasPreviewOnlyModel" class="rounded-[18px] border border-[rgba(245,158,11,0.24)] bg-[rgba(245,158,11,0.08)] px-4 py-3 text-sm text-[rgb(180,83,9)]">
            当前选择了 Yolo26 预留配置。因为后端还没有对应类型，保存按钮会先禁用，避免把不可运行的配置写进真实脚本记录。
          </div>
        </template>

        <template v-else>
          <div class="grid gap-4 xl:grid-cols-[minmax(0,1.3fr)_300px]">
            <SurfacePanel tone="muted" padding="sm" class="space-y-4">
              <div>
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">赞助与联系</p>
                <p class="text-xs text-[var(--app-text-faint)]">和脚本使用者沟通、赞助展示相关的内容集中在这里。</p>
              </div>

              <div class="grid gap-4 md:grid-cols-2">
                <label class="space-y-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">联系方式</span>
                  <input
                    v-model.trim="contactInfoValue"
                    class="app-input"
                    data-testid="script-support-contact-info"
                    maxlength="80"
                    placeholder="QQ / Telegram / Email"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-sm font-medium text-[var(--app-text-strong)]">赞助链接</span>
                  <input
                    v-model.trim="sponsorshipUrlValue"
                    class="app-input"
                    data-testid="script-support-sponsorship-url"
                    maxlength="160"
                    placeholder="https://..."
                  />
                </label>
              </div>
            </SurfacePanel>

            <SurfacePanel tone="muted" padding="sm" class="space-y-4">
              <div>
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">二维码与展示</p>
                <p class="text-xs text-[var(--app-text-faint)]">把二维码预览和赞助信息放在同一视野里，减少来回切换。</p>
              </div>

              <div class="rounded-[16px] border border-[var(--app-border)] px-4 py-3 text-sm">
                <p class="text-xs text-[var(--app-text-faint)]">联系方式预览</p>
                <p class="mt-1 text-[var(--app-text-strong)]">{{ contactInfoValue || '未设置' }}</p>
              </div>

              <div class="rounded-[16px] border border-[var(--app-border)] px-4 py-3 text-sm">
                <p class="text-xs text-[var(--app-text-faint)]">赞助链接预览</p>
                <p class="mt-1 break-all text-[var(--app-text-strong)]">{{ sponsorshipUrlValue || '未设置' }}</p>
              </div>
              <SponsorshipQrField
                v-model="sponsorshipQrValue"
                clear-button-test-id="script-support-sponsorship-qr-clear"
                input-test-id="script-support-sponsorship-qr-input"
                preview-test-id="script-support-sponsorship-qr-preview"
                source-test-id="script-support-sponsorship-qr-source"
              />
            </SurfacePanel>
          </div>
        </template>

        <div class="flex justify-end gap-3">
          <button class="app-button app-button-ghost" type="button" @click="$emit('close')">取消</button>
          <button class="app-button app-button-primary" data-testid="script-submit" type="submit" :disabled="!canSubmit">
            {{ mode === 'edit' ? '保存信息' : '创建脚本' }}
          </button>
        </div>
      </div>
    </form>
  </AppDialog>
</template>

<script setup lang="ts">
import { computed, ref, toRaw, watch } from 'vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { BaseModel } from '@/types/bindings/BaseModel';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { PaddleDetDbNet } from '@/types/bindings/PaddleDetDbNet';
import type { PaddleRecCrnn } from '@/types/bindings/PaddleRecCrnn';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import type { ScriptTableRecord } from '@/types/app/domain';
import type { YoloDet } from '@/types/bindings/YoloDet';
import ModelBaseFields from '@/views/script-list/script-info/ModelBaseFields.vue';
import SponsorshipQrField from '@/views/script-list/script-info/SponsorshipQrField.vue';

type DialogTab = 'basic' | 'models' | 'support';
type DetectorKind = 'none' | 'Yolo11' | 'PaddleDbNet' | 'Yolo26';
type RecognizerKind = 'none' | 'PaddleCrnn';
type EditableDetectorField = 'imgDetModel' | 'txtDetModel';

interface Yolo26Draft {
  baseModel: BaseModel;
  classCount: number;
  confidenceThresh: number;
  iouThresh: number;
  labelPath: string;
  txtIdx: number | null;
}

const props = defineProps<{
  open: boolean;
  mode: 'create' | 'edit';
  script: ScriptTableRecord | null;
}>();

const emit = defineEmits(['close', 'save']);

const tabs = [
  { id: 'basic' as const, label: '基本信息', description: '名称、描述、运行时、版本。' },
  { id: 'models' as const, label: '模型信息', description: '图像检测、文本检测、文本识别。' },
  { id: 'support' as const, label: '赞助信息', description: '联系方式、赞助链接、二维码。' },
];

const runtimeOptions = [
  { label: 'Rhai', value: 'rhai', description: '适合轻量流程和原生脚本。' },
  { label: 'JavaScript', value: 'javaScript', description: '适合复杂逻辑和工具链扩展。' },
  { label: 'Lua', value: 'lua', description: '适合轻量运行时脚本。' },
  { label: 'AI + Vision', value: 'aIAndVision', description: '适合依赖 OCR 与视觉判断的脚本。' },
  { label: 'AI', value: 'aI', description: '适合纯 AI 处理流程。' },
];

const detectorOptions = [
  { label: '不设置', value: 'none', description: '当前字段留空，不启用该类模型。' },
  { label: 'YOLO11', value: 'Yolo11', description: '通用目标检测方案。' },
  { label: 'Paddle DBNet', value: 'PaddleDbNet', description: '适合文本区域检测。' },
  { label: 'YOLO26 (预留)', value: 'Yolo26', description: '仅做前端预留，后端暂未接入。' },
];

const recognizerOptions = [
  { label: '不设置', value: 'none', description: '当前字段留空，不启用识别模型。' },
  { label: 'Paddle CRNN', value: 'PaddleCrnn', description: '当前绑定里可用的文本识别模型。' },
];

const activeTab = ref<DialogTab>('basic');
const form = ref<ScriptTableRecord | null>(null);
const imgDetKind = ref<DetectorKind>('none');
const txtDetKind = ref<DetectorKind>('none');
const txtRecKind = ref<RecognizerKind>('none');
const imgYolo26 = ref<Yolo26Draft>(createYolo26Draft(false));
const txtYolo26 = ref<Yolo26Draft>(createYolo26Draft(true));

function createBaseModel(modelType: BaseModel['modelType'], width: number, height: number): BaseModel {
  return {
    intraThreadNum: 4,
    intraSpinning: true,
    interThreadNum: 1,
    interSpinning: true,
    executionProvider: 'CPU',
    inputWidth: width,
    inputHeight: height,
    modelSource: 'BuiltIn',
    modelPath: '',
    modelType,
  };
}

function createYoloDet(textMode: boolean): YoloDet {
  return {
    baseModel: createBaseModel('Yolo11', 640, 640),
    classCount: textMode ? 1 : 80,
    confidenceThresh: 0.25,
    iouThresh: 0.45,
    labelPath: null,
    txtIdx: textMode ? 0 : null,
  };
}

function createDbNet(): PaddleDetDbNet {
  return {
    baseModel: createBaseModel('PaddleDet5', 640, 640),
    dbThresh: 0.3,
    dbBoxThresh: 0.5,
    unclipRatio: 1.5,
    useDilation: false,
  };
}

function createCrnn(): PaddleRecCrnn {
  return {
    baseModel: createBaseModel('PaddleCrnn5', 320, 48),
    dictPath: null,
  };
}

function createYolo26Draft(textMode: boolean): Yolo26Draft {
  return {
    baseModel: createBaseModel('Yolo11', 640, 640),
    classCount: textMode ? 1 : 80,
    confidenceThresh: 0.25,
    iouThresh: 0.45,
    labelPath: '',
    txtIdx: textMode ? 0 : null,
  };
}

function resolveDetectorKind(model: DetectorType | null): DetectorKind {
  if (!model) return 'none';
  if ('Yolo11' in model) return 'Yolo11';
  if ('PaddleDbNet' in model) return 'PaddleDbNet';
  return 'none';
}

function resolveRecognizerKind(model: RecognizerType | null): RecognizerKind {
  if (!model) return 'none';
  if ('PaddleCrnn' in model) return 'PaddleCrnn';
  return 'none';
}

function syncKinds() {
  if (!form.value) return;
  imgDetKind.value = resolveDetectorKind(form.value.data.imgDetModel);
  txtDetKind.value = resolveDetectorKind(form.value.data.txtDetModel);
  txtRecKind.value = resolveRecognizerKind(form.value.data.txtRecModel);
}

function setDetectorKind(field: EditableDetectorField, nextValue: string | number | null) {
  if (!form.value) return;

  const kind = (nextValue ?? 'none') as DetectorKind;
  if (field === 'imgDetModel') imgDetKind.value = kind;
  else txtDetKind.value = kind;

  if (kind === 'none' || kind === 'Yolo26') {
    form.value.data[field] = null;
    return;
  }

  form.value.data[field] = kind === 'Yolo11' ? { Yolo11: createYoloDet(field === 'txtDetModel') } : { PaddleDbNet: createDbNet() };
}

function setRecognizerKind(nextValue: string | number | null) {
  if (!form.value) return;
  const kind = (nextValue ?? 'none') as RecognizerKind;
  txtRecKind.value = kind;
  form.value.data.txtRecModel = kind === 'PaddleCrnn' ? { PaddleCrnn: createCrnn() } : null;
}

const scriptTypeLabel = computed(() => (form.value?.data.scriptType === 'published' ? '云端版本' : '本地开发'));
const runtimeLabel = computed(() => runtimeOptions.find((item) => item.value === form.value?.data.runtimeType)?.label || '-');
const hasPreviewOnlyModel = computed(() => imgDetKind.value === 'Yolo26' || txtDetKind.value === 'Yolo26');
const canSubmit = computed(() => Boolean(form.value?.data.name.trim()) && !hasPreviewOnlyModel.value);
const dialogWidthClass = computed(() =>
  props.mode === 'create' ? 'max-w-6xl min-h-[80vh] max-h-[calc(100vh-3rem)] flex flex-col' : 'max-w-6xl',
);
const formClass = computed(() =>
  props.mode === 'create' ? 'min-h-0 flex-1 overflow-y-auto pr-1' : '',
);

const descriptionValue = computed({
  get: () => form.value?.data.description || '',
  set: (value: string) => {
    if (form.value) form.value.data.description = value || null;
  },
});

const pkgNameValue = computed({
  get: () => form.value?.data.pkgName || '',
  set: (value: string) => {
    if (form.value) form.value.data.pkgName = value || null;
  },
});

const contactInfoValue = computed({
  get: () => form.value?.data.contactInfo || '',
  set: (value: string) => {
    if (form.value) form.value.data.contactInfo = value || null;
  },
});

const sponsorshipUrlValue = computed({
  get: () => form.value?.data.sponsorshipUrl || '',
  set: (value: string) => {
    if (form.value) form.value.data.sponsorshipUrl = value || null;
  },
});

const sponsorshipQrValue = computed({
  get: () => form.value?.data.sponsorshipQr || '',
  set: (value: string) => {
    if (form.value) form.value.data.sponsorshipQr = value || null;
  },
});

const labelPathImg = computed({
  get: () => (form.value?.data.imgDetModel && 'Yolo11' in form.value.data.imgDetModel ? form.value.data.imgDetModel.Yolo11.labelPath || '' : ''),
  set: (value: string) => {
    if (form.value?.data.imgDetModel && 'Yolo11' in form.value.data.imgDetModel) form.value.data.imgDetModel.Yolo11.labelPath = value || null;
  },
});

const labelPathTxt = computed({
  get: () => (form.value?.data.txtDetModel && 'Yolo11' in form.value.data.txtDetModel ? form.value.data.txtDetModel.Yolo11.labelPath || '' : ''),
  set: (value: string) => {
    if (form.value?.data.txtDetModel && 'Yolo11' in form.value.data.txtDetModel) form.value.data.txtDetModel.Yolo11.labelPath = value || null;
  },
});

const txtIdxValue = computed({
  get: () => (form.value?.data.txtDetModel && 'Yolo11' in form.value.data.txtDetModel ? form.value.data.txtDetModel.Yolo11.txtIdx ?? 0 : 0),
  set: (value: number) => {
    if (form.value?.data.txtDetModel && 'Yolo11' in form.value.data.txtDetModel) form.value.data.txtDetModel.Yolo11.txtIdx = value;
  },
});

const dictPathValue = computed({
  get: () => (form.value?.data.txtRecModel && 'PaddleCrnn' in form.value.data.txtRecModel ? form.value.data.txtRecModel.PaddleCrnn.dictPath || '' : ''),
  set: (value: string) => {
    if (form.value?.data.txtRecModel && 'PaddleCrnn' in form.value.data.txtRecModel) form.value.data.txtRecModel.PaddleCrnn.dictPath = value || null;
  },
});

function cloneScriptRecord(script: unknown): ScriptTableRecord {
  return JSON.parse(JSON.stringify(toRaw(script))) as ScriptTableRecord;
}

function submit() {
  if (!form.value || !canSubmit.value) return;
  form.value.data.name = form.value.data.name.trim();
  form.value.data.verName = form.value.data.verName.trim() || '0.1.0';
  form.value.data.updateTime = new Date().toISOString();
  const nextScript = cloneScriptRecord(form.value);
  emit('save', nextScript);
}

watch(
  () => [props.open, props.script?.id],
  ([open]) => {
    if (!open || !props.script) return;
    form.value = cloneScriptRecord(props.script);
    activeTab.value = 'basic';
    imgYolo26.value = createYolo26Draft(false);
    txtYolo26.value = createYolo26Draft(true);
    syncKinds();
  },
  { immediate: true },
);
</script>
