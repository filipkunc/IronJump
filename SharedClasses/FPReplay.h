//
//  FPReplay.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 8/7/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPGameProtocols.h"
#import "FPGame.h"

typedef struct
{
    float x, y;
    int textureIndex;
    BOOL isVisible;
} FPReplayItem;

@interface FPReplayFrame : NSObject
{
    FPReplayItem *replayItems;

    float playerRotation;
    float playerAlpha;
    int playerSpeedUpCounter;
    
    CGPoint backgroundOffset;
}

- (id)initFromGame:(FPGame *)game;
- (void)setToGame:(FPGame *)game;

@end

@interface FPReplay : NSObject
{
    NSMutableArray *frames;
    int currentFrameIndex;
}

- (void)addFrameFromGame:(FPGame *)game;
- (BOOL)playFrameToGame:(FPGame *)game;
- (void)resetReplay;

@end
