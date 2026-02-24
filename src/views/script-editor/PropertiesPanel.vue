<template>
  <div class="w-80 border-l border-base-300 flex flex-col bg-base-100 shadow-md z-1 h-full">
    <div class="p-3 font-bold text-sm bg-base-200 flex justify-between items-center">
      属性
      <div class="badge badge-sm" v-if="selectedNode">{{ nodeTypeDisplay }}</div>
      <div class="badge badge-sm badge-ghost" v-else>未选择</div>
    </div>

    <!-- Node Selected -->
    <div class="flex-1 p-4 overflow-y-auto" v-if="selectedNode">
      <!-- Common: Label/Remark -->
      <div class="form-control w-full">
        <label class="label"><span class="label-text font-bold">Label (Remark)</span></label>
        <input
          type="text"
          v-model="nodeLabel"
          class="input input-bordered w-full input-sm"
          placeholder="Enter a description..."
          @input="updateLabel"
        />
        <label class="label"><span class="label-text-alt opacity-60">Displayed on the node</span></label>
      </div>

      <div class="divider text-xs opacity-50">Configuration</div>

      <!-- Type: Click -->
      <div v-if="selectedNode.data?.type === 'click'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Target Type</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.targetType">
            <option value="coordinates">Coordinates (x, y)</option>
            <option value="image">Image Match</option>
            <option value="text">OCR Text</option>
          </select>
        </div>

        <div class="form-control w-full" v-if="localData.targetType === 'coordinates'">
          <label class="label"><span class="label-text">Coordinates</span></label>
          <div class="join w-full">
            <input
              type="number"
              v-model="localData.x"
              placeholder="X"
              class="input input-bordered input-sm join-item w-1/2"
            />
            <input
              type="number"
              v-model="localData.y"
              placeholder="Y"
              class="input input-bordered input-sm join-item w-1/2"
            />
          </div>
        </div>

        <div class="form-control w-full" v-else>
          <label class="label"><span class="label-text">Target</span></label>
          <input
            type="text"
            v-model="localData.target"
            placeholder="Image path or text..."
            class="input input-bordered input-sm w-full"
          />
        </div>
      </div>

      <!-- Type: Wait -->
      <div v-if="selectedNode.data?.type === 'wait'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Duration (ms)</span></label>
          <input
            type="number"
            v-model="localData.duration"
            class="input input-bordered input-sm w-full"
            min="100"
            step="100"
          />
        </div>
        <div class="form-control w-full">
          <label class="label cursor-pointer justify-start gap-2">
            <input type="checkbox" v-model="localData.randomize" class="checkbox checkbox-sm" />
            <span class="label-text">Randomize (±20%)</span>
          </label>
        </div>
      </div>

      <!-- Type: Swipe -->
      <div v-if="selectedNode.data?.type === 'swipe'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Start Point (x, y)</span></label>
          <div class="join w-full">
            <input
              type="number"
              v-model="localData.startX"
              placeholder="X"
              class="input input-bordered input-sm join-item w-1/2"
            />
            <input
              type="number"
              v-model="localData.startY"
              placeholder="Y"
              class="input input-bordered input-sm join-item w-1/2"
            />
          </div>
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">End Point (x, y)</span></label>
          <div class="join w-full">
            <input
              type="number"
              v-model="localData.endX"
              placeholder="X"
              class="input input-bordered input-sm join-item w-1/2"
            />
            <input
              type="number"
              v-model="localData.endY"
              placeholder="Y"
              class="input input-bordered input-sm join-item w-1/2"
            />
          </div>
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Duration (ms)</span></label>
          <input type="number" v-model="localData.duration" class="input input-bordered input-sm w-full" min="100" />
        </div>
      </div>

      <!-- Type: Any FlowNode with Condition (IF, While, For) -->
      <div v-if="['if', 'while', 'for'].includes(selectedNode.data?.type)" class="space-y-4">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Condition Type</span></label>
          <select
            class="select select-bordered select-sm w-full"
            v-model="localData.con.type"
            @change="onConTypeChange"
          >
            <option value="rawExpr">Raw Expression (rhai)</option>
            <option value="execNumCompare">Execute Count Compare</option>
            <option value="taskStatus">Task Status Check</option>
            <option value="colorCompare">OCR Color Compare</option>
            <option value="varCompare">Variable Compare</option>
          </select>
        </div>

        <!-- RawExpr -->
        <div v-if="localData.con?.type === 'rawExpr'" class="form-control w-full">
          <label class="label"><span class="label-text">Rhai Expression</span></label>
          <input
            type="text"
            v-model="localData.con.expr"
            placeholder="e.g. var_name == 1"
            class="input input-sm input-bordered w-full font-mono"
          />
        </div>

        <!-- ExecNumCompare -->
        <div v-if="localData.con?.type === 'execNumCompare'" class="space-y-2">
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Target Type</span></label>
            <select class="select select-bordered select-sm w-full" v-model="localData.con.a.type">
              <option value="policy">Policy</option>
              <option value="task">Task</option>
            </select>
          </div>
          <div class="form-control w-full" v-if="localData.con.a?.type">
            <label class="label"><span class="label-text">Target ID</span></label>
            <input type="number" v-model="localData.con.a.id" class="input input-sm input-bordered w-full" />
          </div>
        </div>

        <!-- TaskStatus -->
        <div v-if="localData.con?.type === 'taskStatus'" class="space-y-2">
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Target Action</span></label>
            <select class="select select-bordered select-sm w-full" v-model="localData.con.a.type">
              <option value="getState">Get State</option>
              <option value="setState">Set State</option>
            </select>
          </div>
          <div class="form-control w-full flex gap-2">
            <div class="flex-1">
              <label class="label"><span class="label-text text-xs">Target</span></label>
              <select class="select select-bordered select-sm w-full" v-model="localData.con.a.target.type">
                <option value="policy">Policy</option>
                <option value="task">Task</option>
              </select>
            </div>
            <div class="flex-1">
              <label class="label"><span class="label-text text-xs">ID</span></label>
              <input type="number" v-model="localData.con.a.target.id" class="input input-sm input-bordered w-full" />
            </div>
          </div>
          <div class="form-control w-full flex gap-2">
            <div class="flex-1">
              <label class="label"><span class="label-text text-xs">Status Check</span></label>
              <select class="select select-bordered select-sm w-full" v-model="localData.con.a.status.type">
                <option value="skip">Skip</option>
                <option value="done">Done</option>
              </select>
            </div>
            <div class="flex-1 flex items-end pb-1 pl-2">
              <label class="cursor-pointer label justify-start gap-2 h-8">
                <input type="checkbox" v-model="localData.con.a.status.value" class="checkbox checkbox-sm" />
                <span class="label-text text-xs">Value</span>
              </label>
            </div>
          </div>
        </div>

        <!-- ColorCompare -->
        <div v-if="localData.con?.type === 'colorCompare'" class="space-y-2">
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Target Text</span></label>
            <input
              type="text"
              v-model="localData.con.txtTarget"
              placeholder="Text to match..."
              class="input input-sm input-bordered w-full"
            />
          </div>
          <div class="form-control w-full">
            <label class="cursor-pointer label justify-start gap-2">
              <input type="checkbox" v-model="localData.con.isFont" class="checkbox checkbox-sm" />
              <span class="label-text">Is Font Color (uncheck for Background)</span>
            </label>
          </div>
          <div class="form-control w-full">
            <label class="label"><span class="label-text">RGB Values</span></label>
            <div class="join w-full">
              <input
                type="number"
                v-model="localData.con.r"
                min="0"
                max="255"
                placeholder="R"
                class="join-item input input-sm input-bordered w-1/3"
              />
              <input
                type="number"
                v-model="localData.con.g"
                min="0"
                max="255"
                placeholder="G"
                class="join-item input input-sm input-bordered w-1/3"
              />
              <input
                type="number"
                v-model="localData.con.b"
                min="0"
                max="255"
                placeholder="B"
                class="join-item input input-sm input-bordered w-1/3"
              />
            </div>
          </div>
        </div>

        <!-- VarCompare -->
        <div v-if="localData.con?.type === 'varCompare'" class="space-y-2">
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Variable Name</span></label>
            <input type="text" v-model="localData.con.varName" class="input input-sm input-bordered w-full" />
          </div>
          <div class="flex gap-2 w-full">
            <div class="form-control flex-1">
              <label class="label"><span class="label-text text-xs">Operator</span></label>
              <select class="select select-bordered select-sm w-full" v-model="localData.con.op">
                <option value="eq">==</option>
                <option value="ne">!=</option>
                <option value="lt">&lt;</option>
                <option value="le">&lt;=</option>
                <option value="gt">&gt;</option>
                <option value="ge">&gt;=</option>
              </select>
            </div>
            <div class="form-control flex-1">
              <label class="label"><span class="label-text text-xs">Value Type</span></label>
              <select
                class="select select-bordered select-sm w-full"
                v-model="localData.con.value.type"
                @change="onVarCompareTypeChange"
              >
                <option value="int">Integer</option>
                <option value="float">Float</option>
                <option value="bool">Boolean</option>
                <option value="string">String</option>
              </select>
            </div>
          </div>
          <div class="form-control w-full">
            <label class="label"><span class="label-text text-xs">Value</span></label>
            <div v-if="localData.con.value?.type === 'int'">
              <input
                type="number"
                v-model="localData.con.value.value"
                step="1"
                class="input input-sm input-bordered w-full"
              />
            </div>
            <div v-else-if="localData.con.value?.type === 'float'">
              <input
                type="number"
                v-model="localData.con.value.value"
                step="0.01"
                class="input input-sm input-bordered w-full"
              />
            </div>
            <div v-else-if="localData.con.value?.type === 'bool'">
              <label class="cursor-pointer label justify-start gap-2 h-8 pl-2">
                <input type="checkbox" v-model="localData.con.value.value" class="checkbox checkbox-sm" />
                <span class="label-text text-xs">True / False</span>
              </label>
            </div>
            <div v-else-if="localData.con.value?.type === 'string'">
              <input type="text" v-model="localData.con.value.value" class="input input-sm input-bordered w-full" />
            </div>
          </div>
        </div>
      </div>

      <!-- Type: Find Image -->
      <div v-if="selectedNode.data?.type === 'detect'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Template Image</span></label>
          <div class="join w-full">
            <input
              type="text"
              v-model="localData.imagePath"
              placeholder="Select image..."
              class="input input-bordered input-sm join-item flex-1"
              readonly
            />
            <button class="btn btn-sm join-item btn-primary">Browse</button>
          </div>
        </div>

        <div class="form-control w-full">
          <label class="label"><span class="label-text">Confidence Threshold (%)</span></label>
          <input type="range" v-model="localData.confidence" min="50" max="100" class="range range-sm range-primary" />
          <div class="text-right text-xs opacity-60">{{ localData.confidence || 80 }}%</div>
        </div>

        <div class="form-control w-full">
          <label class="label"><span class="label-text">Store Result In</span></label>
          <input
            type="text"
            v-model="localData.resultVar"
            placeholder="Variable name..."
            class="input input-bordered input-sm w-full font-mono"
          />
        </div>
      </div>

      <!-- Type: OCR -->
      <div v-if="selectedNode.data?.type === 'ocr'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Region (optional)</span></label>
          <div class="grid grid-cols-2 gap-2">
            <input type="number" v-model="localData.regionX" placeholder="X" class="input input-bordered input-sm" />
            <input type="number" v-model="localData.regionY" placeholder="Y" class="input input-bordered input-sm" />
            <input
              type="number"
              v-model="localData.regionW"
              placeholder="Width"
              class="input input-bordered input-sm"
            />
            <input
              type="number"
              v-model="localData.regionH"
              placeholder="Height"
              class="input input-bordered input-sm"
            />
          </div>
        </div>

        <div class="form-control w-full">
          <label class="label"><span class="label-text">Store Result In</span></label>
          <input
            type="text"
            v-model="localData.resultVar"
            placeholder="Variable name..."
            class="input input-bordered input-sm w-full font-mono"
          />
        </div>
      </div>

      <!-- Type: Loop -->
      <div v-if="selectedNode.data?.type === 'loop'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Loop Count</span></label>
          <input type="number" v-model="localData.count" class="input input-bordered input-sm w-full" min="1" />
        </div>

        <div class="form-control w-full">
          <label class="label"><span class="label-text">Loop Type</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.loopType">
            <option value="count">Fixed Count</option>
            <option value="until_found">Until Found</option>
            <option value="until_not_found">Until Not Found</option>
            <option value="infinite">Infinite (with break condition)</option>
          </select>
        </div>

        <div class="form-control w-full" v-if="['until_found', 'until_not_found'].includes(localData.loopType)">
          <label class="label"><span class="label-text">Break Condition</span></label>
          <input
            type="text"
            v-model="localData.breakCondition"
            placeholder="Image or text to find..."
            class="input input-bordered input-sm w-full"
          />
        </div>
      </div>

      <!-- Type: Fallback -->
      <div v-if="selectedNode.data?.type === 'fallback'" class="space-y-3">
        <div class="alert alert-info text-xs">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            class="stroke-current shrink-0 w-4 h-4"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            ></path>
          </svg>
          <span>Fallback executes when all previous conditions fail.</span>
        </div>

        <div class="form-control w-full">
          <label class="label"><span class="label-text">Max Retries</span></label>
          <input
            type="number"
            v-model="localData.maxRetries"
            class="input input-bordered input-sm w-full"
            min="1"
            max="10"
          />
        </div>

        <div class="form-control w-full">
          <label class="label"><span class="label-text">Fallback Actions</span></label>
          <div class="space-y-2">
            <div
              v-for="(strategy, idx) in localData.strategies || []"
              :key="idx"
              class="flex items-center gap-2 p-2 bg-base-200 rounded"
            >
              <span class="badge badge-sm badge-neutral">{{ (idx as number) + 1 }}</span>
              <input
                type="text"
                v-model="strategy.target"
                class="input input-bordered input-xs flex-1"
                placeholder="Target..."
              />
              <select v-model="strategy.action" class="select select-bordered select-xs w-20">
                <option value="click">Click</option>
                <option value="back">Back</option>
              </select>
              <button class="btn btn-xs btn-ghost btn-circle text-error" @click="removeStrategy(idx as number)">
                ×
              </button>
            </div>
          </div>
          <button class="btn btn-xs btn-ghost mt-2" @click="addStrategy">+ Add Action</button>
        </div>
      </div>

      <!-- Type: Screenshot -->
      <div v-if="selectedNode.data?.type === 'screenshot'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Output Variable</span></label>
          <input
            type="text"
            v-model="localData.outputVar"
            placeholder="Variable name..."
            class="input input-bordered input-sm w-full font-mono"
          />
        </div>
      </div>

      <!-- Type: Variable -->
      <div v-if="selectedNode.data?.type === 'variable'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Variable Name</span></label>
          <input
            type="text"
            v-model="localData.varName"
            placeholder="e.g. price_str"
            class="input input-bordered input-sm w-full font-mono"
          />
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Operation</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.opType">
            <option value="set">Set Literal</option>
            <option value="math">Math Expression</option>
            <option value="string">String Slice/Split</option>
            <option value="regex">Regex Extract</option>
          </select>
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Expression / Value</span></label>
          <textarea
            v-model="localData.expression"
            class="textarea textarea-bordered text-xs font-mono h-20"
            placeholder="e.g. input.split('/')[0]"
          ></textarea>
        </div>
      </div>

      <!-- Type: Filter -->
      <div v-if="selectedNode.data?.type === 'filter'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Source Array</span></label>
          <input
            type="text"
            v-model="localData.sourceVar"
            placeholder="Variable name..."
            class="input input-bordered input-sm w-full font-mono"
          />
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Target Variable</span></label>
          <input
            type="text"
            v-model="localData.targetVar"
            placeholder="Default same as source"
            class="input input-bordered input-sm w-full font-mono"
          />
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Mode</span></label>
          <div class="join w-full">
            <button
              class="btn btn-xs join-item flex-1"
              :class="localData.mode === 'filter' ? 'btn-primary' : 'btn-ghost'"
              @click="localData.mode = 'filter'"
            >
              Filter
            </button>
            <button
              class="btn btn-xs join-item flex-1"
              :class="localData.mode === 'map' ? 'btn-primary' : 'btn-ghost'"
              @click="localData.mode = 'map'"
            >
              Map
            </button>
          </div>
        </div>
        <div class="form-control w-full">
          <label class="label"
            ><span class="label-text">{{
              localData.mode === 'filter' ? 'Condition (item)' : 'Logic (item)'
            }}</span></label
          >
          <textarea
            v-model="localData.logic"
            class="textarea textarea-bordered text-xs font-mono h-20"
            :placeholder="localData.mode === 'filter' ? 'item.score > 80' : 'item.name'"
          ></textarea>
        </div>
      </div>

      <!-- Type: Vision Logic (Advanced Search) -->
      <div v-if="selectedNode.data?.type === 'vision_logic'" class="space-y-4">
        <div class="form-control w-full">
          <label class="label pb-1"><span class="label-text font-bold">Search Rule</span></label>

          <div class="bg-base-200 p-3 rounded-lg border border-base-300 space-y-3">
            <!-- Group Settings -->
            <div class="flex items-center gap-2">
              <select class="select select-bordered select-xs w-20" v-model="localData.rule.op">
                <option value="And">AND</option>
                <option value="Or">OR</option>
                <option value="Not">NOT</option>
              </select>
              <select class="select select-bordered select-xs flex-1" v-model="localData.rule.scope">
                <option value="Global">Global Screen</option>
                <option value="Item">Within Element</option>
              </select>
            </div>

            <!-- Items List -->
            <div class="space-y-2">
              <div v-for="(item, idx) in localData.rule.items" :key="idx" class="flex items-center gap-2">
                <div class="flex-1 flex items-center bg-base-100 rounded px-2 py-1 gap-2 border border-base-300">
                  <span class="text-[10px] opacity-50">{{ item.type === 'Keyword' ? 'KW' : 'GRP' }}</span>
                  <input
                    v-if="item.type === 'Keyword'"
                    type="text"
                    v-model="item.text"
                    class="input input-ghost input-xs flex-1 focus:bg-base-100"
                    placeholder="Pattern (Text/OBJ:ID/BG:COLOR)..."
                  />
                  <span v-else class="text-xs italic flex-1">Sub-group logic...</span>
                </div>
                <button class="btn btn-xs btn-ghost btn-circle text-error" @click="removeRuleItem(idx as number)">
                  ×
                </button>
              </div>
            </div>

            <!-- Add Button -->
            <div class="flex gap-2">
              <button class="btn btn-xs btn-outline btn-primary flex-1" @click="addRuleKeyword">+ Keyword</button>
            </div>
          </div>
        </div>

        <div class="form-control w-full">
          <label class="label"><span class="label-text">Output Variable</span></label>
          <input
            type="text"
            v-model="localData.outputVar"
            placeholder="Variable name..."
            class="input input-bordered input-sm w-full font-mono"
          />
          <label class="label pb-0"
            ><span class="label-text-alt opacity-50 text-[10px]">Contains matched hits with coordinates</span></label
          >
        </div>
      </div>

      <!-- Type: SubFlow -->
      <div v-if="selectedNode.data?.type === 'subflow'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Target Task</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.targetTaskId">
            <option :value="null">Select a task...</option>
            <option value="1">Login</option>
            <option value="2">Sign In</option>
            <option value="3">Claim Rewards</option>
          </select>
        </div>

        <div class="form-control w-full">
          <label class="label cursor-pointer justify-start gap-2">
            <input type="checkbox" v-model="localData.waitForComplete" class="checkbox checkbox-sm" />
            <span class="label-text">Wait for completion</span>
          </label>
        </div>
      </div>

      <!-- Common: Advanced Options -->
      <div
        class="collapse collapse-arrow bg-base-200 mt-4"
        v-if="!['start', 'input'].includes(selectedNode.data?.type)"
      >
        <input type="checkbox" />
        <div class="collapse-title text-sm font-medium">Advanced Options</div>
        <div class="collapse-content space-y-3">
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Delay Before (ms)</span></label>
            <input type="number" v-model="localData.delayBefore" class="input input-bordered input-sm w-full" min="0" />
          </div>
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Delay After (ms)</span></label>
            <input type="number" v-model="localData.delayAfter" class="input input-bordered input-sm w-full" min="0" />
          </div>
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Condition (Rhai Script)</span></label>
            <textarea
              v-model="localData.condition"
              class="textarea textarea-bordered text-xs font-mono h-20"
              placeholder="// Return true to execute this node"
            ></textarea>
          </div>
        </div>
      </div>

      <!-- Delete Button -->
      <div class="mt-8" v-if="!['start', 'input'].includes(selectedNode.data?.type)">
        <button class="btn btn-error btn-sm w-full btn-outline" @click="$emit('delete-node')">Delete Node</button>
      </div>
    </div>

    <!-- No Selection -->
    <div class="flex-1 p-10 flex flex-col items-center justify-center text-base-content/30" v-else>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="48"
        height="48"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        <line x1="9" y1="9" x2="15" y2="15" />
        <line x1="15" y1="9" x2="9" y2="15" />
      </svg>
      <span class="mt-2 text-sm">Select a node to edit</span>
      <span class="mt-1 text-xs opacity-50">Click on any node in the canvas</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { Node } from '@vue-flow/core';
import { DEFAULT_FALLBACK_STRATEGIES, getNodeDisplay, NODE_DATA_DEFAULTS } from './config';

const props = defineProps<{
  selectedNode: Node | null;
}>();

const emit = defineEmits<{
  (e: 'delete-node'): void;
  (e: 'update-node', id: string, data: any): void;
}>();

const nodeLabel = ref('');
const localData = ref<any>({});

const nodeTypeDisplay = computed(() => {
  if (!props.selectedNode) return 'Unknown';
  const type = props.selectedNode.data?.type;
  return getNodeDisplay(type) || type || 'Unknown';
});

watch(
  () => props.selectedNode,
  (newNode) => {
    if (newNode) {
      nodeLabel.value = (newNode.label as string) || '';
      localData.value = { ...newNode.data };

      if (!localData.value.targetType) localData.value.targetType = NODE_DATA_DEFAULTS.targetType;
      if (!localData.value.searchType) localData.value.searchType = NODE_DATA_DEFAULTS.searchType;
      if (!localData.value.confidence) localData.value.confidence = NODE_DATA_DEFAULTS.confidence;
      if (!localData.value.duration) localData.value.duration = NODE_DATA_DEFAULTS.duration;
      if (!localData.value.timeout) localData.value.timeout = NODE_DATA_DEFAULTS.timeout;
      if (!localData.value.count) localData.value.count = NODE_DATA_DEFAULTS.count;
      if (!localData.value.loopType) localData.value.loopType = NODE_DATA_DEFAULTS.loopType;
      if (!localData.value.maxRetries) localData.value.maxRetries = NODE_DATA_DEFAULTS.maxRetries;
      if (!localData.value.strategies) localData.value.strategies = DEFAULT_FALLBACK_STRATEGIES.map((s) => ({ ...s }));
      if (localData.value.waitForComplete === undefined)
        localData.value.waitForComplete = NODE_DATA_DEFAULTS.waitForComplete;

      if (['if', 'while', 'for'].includes(newNode.data?.type)) {
        if (!localData.value.con) {
          localData.value.con = { type: 'rawExpr', expr: '' };
        }
      }

      if (newNode.data?.type === 'vision_logic') {
        if (!localData.value.rule) {
          localData.value.rule = { type: 'Group', op: 'And', scope: 'Global', items: [] };
        }
        if (!localData.value.outputVar) localData.value.outputVar = 'search_results';
      }
    }
  },
  { immediate: true, deep: true }
);

const onConTypeChange = () => {
  const t = localData.value.con.type;
  if (t === 'rawExpr') {
    localData.value.con = { type: 'rawExpr', expr: '' };
  } else if (t === 'execNumCompare') {
    localData.value.con = { type: 'execNumCompare', a: { type: 'task', id: 0 } };
  } else if (t === 'taskStatus') {
    localData.value.con = {
      type: 'taskStatus',
      a: {
        type: 'getState',
        target: { type: 'task', id: 0 },
        status: { type: 'done', value: true },
      },
    };
  } else if (t === 'colorCompare') {
    localData.value.con = { type: 'colorCompare', txtTarget: '', isFont: true, r: 0, g: 0, b: 0 };
  } else if (t === 'varCompare') {
    localData.value.con = { type: 'varCompare', varName: '', op: 'eq', value: { type: 'string', value: '' } };
  }
};

const onVarCompareTypeChange = () => {
  const t = localData.value.con.value.type;
  if (t === 'int' || t === 'float') localData.value.con.value.value = 0;
  else if (t === 'bool') localData.value.con.value.value = true;
  else localData.value.con.value.value = '';
};

watch(
  localData,
  (newData) => {
    if (props.selectedNode) {
      emit('update-node', props.selectedNode.id, newData);
    }
  },
  { deep: true }
);

const updateLabel = () => {
  if (props.selectedNode) {
    emit('update-node', props.selectedNode.id, { ...localData.value, label: nodeLabel.value });
  }
};

const addStrategy = () => {
  if (!localData.value.strategies) {
    localData.value.strategies = [];
  }
  localData.value.strategies.push({ target: '', action: 'click' });
};

const removeStrategy = (idx: number) => {
  localData.value.strategies.splice(idx, 1);
};

const addRuleKeyword = () => {
  if (!localData.value.rule.items) localData.value.rule.items = [];
  localData.value.rule.items.push({ type: 'Keyword', text: '' });
};

const removeRuleItem = (idx: number) => {
  localData.value.rule.items.splice(idx, 1);
};
</script>
