//
//  FPStage.h
//  IronJump
//
//  Created by Filip Kunc on 8/14/10.
//  For license see LICENSE.TXT
//

#import "FPGameProtocols.h"

typedef enum
{
    FPMenuButtonNone,
    FPMenuButtonPlay
} FPMenuButton;

@interface FPStage : NSObject 
{
	NSMutableArray *levels;
	int stageNumber;
	BOOL isCustom;
}

@property (readonly, assign) int stageNumber;
@property (readonly, assign) BOOL isCustom;
@property (readonly, assign) NSUInteger count;
@property (readonly, assign) int currentLevelIndex;

+ (void)initStages;
+ (void)saveDefaults;
+ (void)decideCurrentStage;
+ (NSUInteger)stageCount;
+ (void)addStagesFromLevels:(NSArray *)levelNames isCustom:(BOOL)custom;
+ (FPStage *)stageAtIndex:(NSUInteger)index;
+ (FPStage *)currentStage;
+ (FPMenuButton)touchEndedAtPoint:(CGPoint)point;
+ (void)drawCurrentStage;
+ (BOOL)nextLevel;

- (id)initWithStage:(int)aStage isCustom:(BOOL)custom;
- (void)addLevel:(NSString *)level;
- (NSString *)levelAtIndex:(NSUInteger)index;
- (BOOL)touchedOnLevelAtLocation:(CGPoint)location;
- (void)draw;

@end
