<script lang="ts">
    import type {Rule, RuleCondition, ConditionField, ConditionOperator, ConditionLogic} from '$lib/types';

    interface Props {
        rule: Rule;
        onUpdate: (rule: Rule) => void;
    }

    let {rule, onUpdate}: Props = $props();

    const fieldOptions: { value: ConditionField; label: string }[] = [
        {value: 'filename', label: 'File Name'},
        {value: 'extension', label: 'Extension'},
        {value: 'filesize', label: 'File Size'},
        {value: 'created', label: 'Created Date'},
        {value: 'modified', label: 'Modified Date'},
    ];

    const stringOperators: { value: ConditionOperator; label: string }[] = [
        {value: 'equals', label: 'Equals'},
        {value: 'not_equals', label: 'Not Equals'},
        {value: 'contains', label: 'Contains'},
        {value: 'not_contains', label: 'Does Not Contain'},
        {value: 'starts_with', label: 'Starts With'},
        {value: 'ends_with', label: 'Ends With'},
        {value: 'regex_match', label: 'Regex Match'},
    ];

    const numberOperators: { value: ConditionOperator; label: string }[] = [
        {value: 'equals', label: 'Equals'},
        {value: 'not_equals', label: 'Not Equals'},
        {value: 'greater_than', label: 'Greater Than'},
        {value: 'less_than', label: 'Less Than'},
    ];

    function getOperatorOptions(field: ConditionField): { value: ConditionOperator; label: string }[] {
        if (field === 'filesize' || field === 'created' || field === 'modified') {
            return numberOperators;
        }
        return stringOperators;
    }

    function addCondition() {
        const newCondition: RuleCondition = {
            field: 'filename',
            operator: 'contains',
            value: '',
            case_sensitive: false,
        };

        rule.conditions = [...rule.conditions, newCondition];
        onUpdate(rule);
    }

    function removeCondition(index: number) {
        rule.conditions = rule.conditions.filter((_, i) => i !== index);
        onUpdate(rule);
    }

    function updateCondition(index: number, updates: Partial<RuleCondition>) {
        rule.conditions = rule.conditions.map((condition, i) => {
            if (i === index) {
                const updated = {...condition, ...updates};

                // Reset operator if a field type changed
                if (updates.field && updates.field !== condition.field) {
                    const operators = getOperatorOptions(updates.field);
                    updated.operator = operators[0].value;
                }

                return updated;
            }
            return condition;
        });
        onUpdate(rule);
    }

    function handleFieldChange(index: number, value: string) {
        updateCondition(index, { field: value as ConditionField });
    }

    function handleOperatorChange(index: number, value: string) {
        updateCondition(index, { operator: value as ConditionOperator });
    }

    function updateConditionLogic(logic: ConditionLogic) {
        rule.condition_logic = logic;
        onUpdate(rule);
    }

    function getValuePlaceholder(field: ConditionField): string {
        switch (field) {
            case 'filename':
                return 'e.g., invoice, report';
            case 'extension':
                return 'e.g., pdf, docx';
            case 'filesize':
                return 'Size in bytes, e.g., 1048576 (1MB)';
            case 'created':
            case 'modified':
                return 'Unix timestamp, e.g., 1609459200';
            default:
                return '';
        }
    }
</script>

<div class="space-y-4">
    <div class="flex items-center justify-between">
        <div>
            <h4 class="font-medium">Conditions</h4>
            <p class="text-xs text-muted-foreground">
                Files must match {rule.conditions.length > 0 ? 'these conditions' : 'any condition'}
            </p>
        </div>
        <button
                onclick={addCondition}
                class="px-3 py-1 text-sm bg-secondary text-secondary-foreground rounded hover:opacity-90"
        >
            + Add Condition
        </button>
    </div>

    {#if rule.conditions.length > 1}
        <div class="flex items-center gap-2">
            <span class="text-sm text-muted-foreground">Match:</span>
            <div class="flex gap-1">
                <button
                        onclick={() => updateConditionLogic('AND')}
                        class="px-3 py-1 text-sm rounded transition-colors {
            rule.condition_logic === 'AND'
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted text-muted-foreground hover:bg-secondary'
          }"
                >
                    All (AND)
                </button>
                <button
                        onclick={() => updateConditionLogic('OR')}
                        class="px-3 py-1 text-sm rounded transition-colors {
            rule.condition_logic === 'OR'
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted text-muted-foreground hover:bg-secondary'
          }"
                >
                    Any (OR)
                </button>
            </div>
        </div>
    {/if}

    {#if rule.conditions.length === 0}
        <div class="p-4 border border-dashed border-border rounded text-center text-sm text-muted-foreground">
            No conditions added. This rule will match all files.
        </div>
    {:else}
        <div class="space-y-3">
            {#each rule.conditions as condition, index (index)}
                <div class="p-3 border border-border rounded-lg space-y-3">
                    <div class="flex items-center gap-2">
                        <div class="flex-1 grid grid-cols-3 gap-2">
                            <!-- Field Selection -->
                            <div>
                                <label for="condition-field-{index}" class="text-xs text-muted-foreground">Field</label>
                                <select
                                        id="condition-field-{index}"
                                        onchange={(e) => handleFieldChange(index, e.currentTarget.value)}
                                        class="w-full px-2 py-1 text-sm rounded border border-input bg-background"
                                >
                                    {#each fieldOptions as option}
                                        <option value={option.value} selected={option.value === condition.field}>{option.label}</option>
                                    {/each}
                                </select>
                            </div>

                            <!-- Operator Selection -->
                            <div>
                                <label for="condition-operator-{index}" class="text-xs text-muted-foreground">Operator</label>
                                <select
                                        id="condition-operator-{index}"
                                        onchange={(e) => handleOperatorChange(index, e.currentTarget.value)}
                                        class="w-full px-2 py-1 text-sm rounded border border-input bg-background"
                                >
                                    {#each getOperatorOptions(condition.field) as option}
                                        <option value={option.value} selected={option.value === condition.operator}>{option.label}</option>
                                    {/each}
                                </select>
                            </div>

                            <!-- Value Input -->
                            <div>
                                <label for="condition-value-{index}" class="text-xs text-muted-foreground">Value</label>
                                <input
                                        id="condition-value-{index}"
                                        type="text"
                                        value={condition.value}
                                        oninput={(e) => updateCondition(index, { value: e.currentTarget.value })}
                                        placeholder={getValuePlaceholder(condition.field)}
                                        class="w-full px-2 py-1 text-sm rounded border border-input bg-background"
                                />
                            </div>
                        </div>

                        <button
                                onclick={() => removeCondition(index)}
                                class="text-destructive hover:underline text-sm mt-4"
                        >
                            Remove
                        </button>
                    </div>

                    <!-- Case Sensitivity (only for string fields) -->
                    {#if condition.field === 'filename' || condition.field === 'extension'}
                        <label class="flex items-center gap-2">
                            <input
                                    type="checkbox"
                                    checked={condition.case_sensitive}
                                    onchange={(e) => updateCondition(index, { case_sensitive: e.currentTarget.checked })}
                                    class="rounded"
                            />
                            <span class="text-xs text-muted-foreground">Case sensitive</span>
                        </label>
                    {/if}

                    <!-- Helper text based on field type -->
                    <div class="text-xs text-muted-foreground">
                        {#if condition.field === 'filesize'}
                            1 MB = 1,048,576 bytes, 1 GB = 1,073,741,824 bytes
                        {:else if condition.field === 'created' || condition.field === 'modified'}
                            Use Unix timestamp (seconds since Jan 1, 1970)
                        {:else if condition.field === 'extension'}
                            Don't include the dot (e.g., 'pdf' not '.pdf')
                        {:else if condition.operator === 'regex_match'}
                            Use regular expression pattern (e.g., '^invoice_\d{'{4}'}$')
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
