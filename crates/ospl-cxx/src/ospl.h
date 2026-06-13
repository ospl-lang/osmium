#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef union RV
{
    intptr_t    integer;
    uintptr_t   address;
    double      float64;
    bool        boolean;
    uint32_t    character;
    uint32_t*   str;
    void*       list;
    void*       func;
    void*       scope;
    uint32_t    foreign;
    int         nothing[0];
};

typedef struct {
    uint8_t tag;
    RV value;
} RuntimeValue;

typedef void cVM;

typedef void cInst;

cVM *OSPL_vm_create(void);
void OSPL_vm_destroy(cVM *vm);
void OSPL_vm_run(cVM *vm, cInst *i);
RuntimeValue *OSPL_value_new_str(const char *v);
RuntimeValue *OSPL_value_new_undefined(void);
RuntimeValue *OSPL_value_new_nul(void);
RuntimeValue *OSPL_value_new_char(uint32_t c);
cInst *OSPL_inst_new(void);
void OSPL_inst_destroy(cInst *i);
void OSPL_inst_set_opcode(cInst *b, uint8_t o);
void OSPL_inst_add_index(cInst *b, uintptr_t i);
void OSPL_inst_add_child(cInst *b, uintptr_t num, const cInst *data);
uint8_t OSPL_inst_get_opcode(cInst *b);